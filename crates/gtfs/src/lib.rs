use data_model::agency::{Agency, AgencyId};
use database::{GtfsDatabase, InMemoryPrimaryKeyTable, PrimaryKeyTable};
use reqwest;
use reqwest::cookie::Jar;
use std::fs::{self, File};
use std::io::{self, copy};
use std::path::Path;
use std::sync::Arc;
use std::{error::Error, io::Cursor};

pub mod collector;
pub mod data_model;
pub mod database;
pub mod domain_model;
pub mod realtime;
mod serde;

pub mod sources {
    /// # Deutschland gesamt
    ///
    /// Kompletter ÖV in Deutschland.
    pub const GERMANY_ALL: &str = "https://download.gtfs.de/germany/free/latest.zip";

    /// # Schienenfernverkehr Deutschland
    ///
    /// Kompletter Schienenfernverkehr der Deutschen Bahn und anderer Anbieter
    /// (ICE, IC, ECE, EC, EN, railJet, Nachtzüge) in Deutschland.
    pub const GERMANY_LONG_DISTANCE_RAIL_TRANSPORT: &str =
        "https://download.gtfs.de/germany/fv_free/latest.zip";

    /// # Öffentlicher Nahverkehr Deutschland
    ///
    /// Kompletter ÖPNV in Deutschland, inklusive U-Bahnen, Stadtbahnen,
    /// Straßenbahnen, Fähren, Zahnradbahnen und Busse sämtlicher Verkehrsverbünde
    /// und Anbieter.
    pub const GERMANY_PUBLIC_LOCAL_TRANSPORT: &str =
        "https://download.gtfs.de/germany/nv_free/latest.zip";

    /// # Schienenregionalverkehr Deutschland
    ///
    /// Kompletter Schienenregionalverkehr der Deutschen Bahn und anderer Anbieter
    /// (RB, RE, IRE, S-Bahnen, nicht-bundeseigene Bahnen) in Deutschland.
    pub const GERMANY_REGIONAL_RAIL_TRANSPORT: &str =
        "https://download.gtfs.de/germany/rv_free/latest.zip";

    pub const GERMANY_REALTIME: &str = "https://realtime.gtfs.de/realtime-free.pb";
}

pub fn open_database() -> Result<GtfsDatabase, Box<dyn Error>> {
    let file_path = "resources/fahrplan_sh/agency.txt";
    let file = File::open(file_path)?;
    let mut reader = csv::Reader::from_reader(file);
    let mut agencies: InMemoryPrimaryKeyTable<Option<AgencyId>, Agency> =
        InMemoryPrimaryKeyTable::new();
    for row in reader.deserialize() {
        let agency: Agency = row?;
        agencies.insert(agency)
    }
    Ok(GtfsDatabase {
        agency: Box::new(agencies),
        stops: Box::new(InMemoryPrimaryKeyTable::new()),
        routes: Box::new(InMemoryPrimaryKeyTable::new()),
        trips: Box::new(InMemoryPrimaryKeyTable::new()),
        stop_times: Box::new(InMemoryPrimaryKeyTable::new()),
    })
}

pub async fn download_gtfs(url: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let zip_name = "latest.zip";
    download_file(url, zip_name).await?;
    extract_zip(zip_name)?;
    Ok(())
}

pub async fn download_file(
    url: &str,
    file_name: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let jar = Arc::new(Jar::default());

    let client = reqwest::Client::builder()
        .cookie_provider(Arc::clone(&jar))
        .build()?;

    let response = client.get(url).send().await?;

    let mut file = std::fs::File::create(file_name)?;
    let mut content = Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    Ok(())
}

fn extract_zip(filename: &str) -> Result<(), io::Error> {
    let fname = Path::new(filename);
    let file = File::open(&fname)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue, // Skip to the next file if the path is None.
        };

        let comment = file.comment();
        if !comment.is_empty() {
            println!("File {} comment: {}", i, comment); // Print the file comment if it's not empty.
        }

        if file.name().ends_with('/') {
            println!("File {} extracted to \"{}\"", i, outpath.display()); // Print a message indicating the directory extraction.
            fs::create_dir_all(&outpath)?; // Create the directory.
        } else {
            println!(
                "File {} extracted to \"{}\" ({} bytes)",
                i,
                outpath.display(),
                file.size()
            );

            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p)?;
                }
            }

            let mut outfile = File::create(&outpath)?;
            copy(&mut file, &mut outfile)?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}
