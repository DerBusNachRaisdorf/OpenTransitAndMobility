use std::num::ParseIntError;

use serde::{Deserialize, Serialize};

pub mod agency;
pub mod calendar;
pub mod calendar_dates;
pub mod frequencies;
pub mod routes;
pub mod shapes;
pub mod stop_times;
pub mod stops;
pub mod transfers;
pub mod trips;

pub mod realtime {
    include!(concat!(env!("OUT_DIR"), "/protobuf/transit_realtime.rs"));
}

// TODO: set type aliases to apropriate types.
// TODO: move some of this types into `utility` crate and only keep
//       serialize / deserialize modules here?

/// A color encoded as a six-digit hexadecimal number.
/// Refer to https://htmlcolorcodes.com to generate a valid value
/// (the leading "#" must not be included).
///
/// # Examples
///
/// `FFFFFF` for white, `000000` for black or `0039A6` for the A,C,E lines in NYMTA.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub fn from_hex(hex: &str) -> Option<Self> {
        let rgb_strings = if hex.len() == 3 {
            [
                hex[0..1].repeat(2),
                hex[1..2].repeat(2),
                hex[2..3].repeat(2),
            ]
        } else if hex.len() == 6 {
            [
                hex[0..2].to_owned(),
                hex[2..4].to_owned(),
                hex[4..6].to_owned(),
            ]
        } else {
            return None;
        };
        let rgb = rgb_strings
            .iter()
            .map(|val| u8::from_str_radix(val, 16))
            .collect::<Result<Vec<u8>, ParseIntError>>()
            .ok()?;
        assert_eq!(rgb.len(), 3);
        Some(Color {
            red: rgb[0],
            green: rgb[1],
            blue: rgb[2],
        })
    }

    pub fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    pub fn white() -> Self {
        Self::from_rgb(255, 255, 255)
    }

    pub fn black() -> Self {
        Self::from_rgb(0, 0, 0)
    }

    pub fn to_hex(&self) -> String {
        format!("{:02X}{:02X}{:02X}", self.red, self.green, self.blue)
    }
}

/// An ISO 4217 alphabetical currency code. For the list of current currency, refer
/// to https://en.wikipedia.org/wiki/ISO_4217#Active_codes.
///
/// # Examples
///
/// `CAD` for Canadian dollars, `EUR` for euros or `JPY` for Japanese yen.
pub type CurrencyCode = String;

/// A decimal value indicating a currency amount. The number of decimal places is
/// specified by ISO 4217 for the accompanying Currency code. All financial
/// calculations should be processed as decimal, currency, or another equivalent type
/// suitable for financial calculations depending on the programming language used to
/// consume data. Processing currency amounts as float is discouraged due to gains or
/// losses of money during calculations.
pub type CurrencyAmount = String;

/// Service day in the YYYYMMDD format. Since time within a service day may be above
/// 24:00:00, a service day may contain information for the subsequent day(s).
///
/// # Examples
///
/// `20180913` for September 13th, 2018.
#[allow(dead_code)]
type Date = String;

/// An email address.
///
/// # Examples
///
/// `example@example.com`
type Email = String;

/// An ID field value is an internal ID, not intended to be shown to riders, and is a
/// sequence of any UTF-8 characters. Using only printable ASCII characters is
/// recommended. An ID is labeled "unique ID" when it must be unique within a file.
/// IDs defined in one .txt file are often referenced in another .txt file. IDs that
/// reference an ID in another table are labeled "foreign ID".
///
/// # Examples
///
/// The `stop_id` field in stops.txt is a "unique ID". The `parent_station` field in
/// stops.txt is a "foreign ID referencing `stops.stop_id`".
pub type IdString = String;

/// An IETF BCP 47 language code. For an introduction to IETF BCP 47, refer to http://www.rfc-editor.org/rfc/bcp/bcp47.txt and http://www.w3.org/International/articles/language-tags/.
///
/// # Examples
///
/// `en` for English, `en-US` for American English or `de` for German.
pub type LanguageCode = String;

/// WGS84 latitude in decimal degrees. The value must be greater than or equal to
/// -90.0 and less than or equal to 90.0.
///
/// # Examples
///
/// `41.890169` for the Colosseum in Rome.
pub type Latitude = f64;

/// WGS84 longitude in decimal degrees. The value must be greater than or equal to
/// -180.0 and less than or equal to 180.0.
///
/// # Examples
///
/// `12.492269` for the Colosseum in Rome.
pub type Longitude = f64;

/// A phone number.
pub type PhoneNumber = String;

/// Time in the HH:MM:SS format (H:MM:SS is also accepted). The time is measured from
/// "noon minus 12h" of the service day (effectively midnight except for days on
/// which daylight savings time changes occur). For times occurring after midnight
/// on the service day, enter the time as a value greater than 24:00:00 in HH:MM:SS.
///
/// # Examples
///
/// `14:30:00` for 2:30PM or `25:35:00` for 1:35AM on the next day.
pub type Time = String;

/// TZ timezone from the https://www.iana.org/time-zones. Timezone names never contain
/// the space character but may contain an underscore.
/// Refer to http://en.wikipedia.org/wiki/List_of_tz_zones for a list of valid values.
///
/// # Examples
///
/// `Asia/Tokyo`, `America/Los_Angeles` or `Africa/Cairo`.
pub type Timezone = String;

/// A fully qualified URL that includes http:// or https://, and any special
/// characters in the URL must be correctly escaped.
/// See the following http://www.w3.org/Addressing/URL/4_URI_Recommentations.html for
/// a description of how to create fully qualified URL values.
pub type Url = String;
