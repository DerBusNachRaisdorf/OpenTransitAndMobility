use serde::{Deserialize, Deserializer, Serialize};

pub mod timetables;
pub mod station_data;

/// WIP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TripCategory {
    ICE,
    RE,
    Other(String),
}

pub mod timestamp {
    use chrono::{DateTime, Local, TimeZone};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const BAHN_FORMAT: &'static str = "%y%m%d%H%M";
    const USUAL_FORMAT: &'static str = "%Y-%m-%d %H:%M";

    pub fn serialize<S>(
        date: &DateTime<Local>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let s = format!("{}", date.format(USUAL_FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<DateTime<Local>, D::Error>
        where
            D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        //Local.datetime_from_str(&s, BAHN_FORMAT).map_err(serde::de::Error::custom)

        /* support both - bahn timestamp and a ususal datetime format */
        Local.datetime_from_str(&s, BAHN_FORMAT)            // try parse as Bahn Timestamp...
            .or(Local.datetime_from_str(&s, USUAL_FORMAT))  // ... if that failed, try parse as usual time format...
            .map_err(serde::de::Error::custom)
    }
}

pub mod timestamp_opt {
    use chrono::{DateTime, Local, TimeZone};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const BAHN_FORMAT: &'static str = "%y%m%d%H%M";
    const USUAL_FORMAT: &'static str = "%Y-%m-%d %H:%M";

    pub fn serialize<S>(
        date: &Option<DateTime<Local>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        match date {
            Some(d) => {
                let s = format!("{}", d.format(USUAL_FORMAT));
                serializer.serialize_str(&s)
            }
            None => serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Option<DateTime<Local>>, D::Error>
        where
            D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        /*
        match Local.datetime_from_str(&s, BAHN_FORMAT).map_err(serde::de::Error::custom) {
            Ok(o) => Ok(Some(o)),
            Err(e) => Err(e)
        }
        */

        // TODO: use `.map(|datetime| Some(datetime))` instead of match block.
        /* support both - the bahn timestamp and the usual datetime format */
        match Local.datetime_from_str(&s, BAHN_FORMAT)
            .or(Local.datetime_from_str(&s, USUAL_FORMAT))
            .map_err(serde::de::Error::custom)
        {
            Ok(o) => Ok(Some(o)),
            Err(e) => Err(e)
        }
    }
}

pub fn deserialize_path<'de, D>(
    deserializer: D,
) -> Result<Vec<String>, D::Error>
    where
        D: Deserializer<'de>,
{
    Ok(String::deserialize(deserializer)?
        .split('|')
        .filter(|stop| !stop.is_empty())
        .map(str::to_owned)
        .collect())
}

pub fn deserialize_path_opt<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<String>>, D::Error>
    where
        D: Deserializer<'de>,
{
    Ok(Some(deserialize_path(deserializer)?))
}
