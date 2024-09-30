use chrono::Duration;
use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug)]
struct SerializableDuration(Duration);

impl Serialize for SerializableDuration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Berechne Stunden, Minuten und Sekunden
        let total_seconds = self.0.num_seconds();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        // Formatieren als hh:mm:ss
        let formatted = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);
        serializer.serialize_str(&formatted)
    }
}

impl<'de> Deserialize<'de> for SerializableDuration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.split(':').collect();

        if parts.len() != 3 {
            return Err(D::Error::invalid_length(
                parts.len(),
                &"Expected format hh:mm:ss",
            ));
        }

        let hours: i64 = parts[0].parse().map_err(D::Error::custom)?;
        let minutes: i64 = parts[1].parse().map_err(D::Error::custom)?;
        let seconds: i64 = parts[2].parse().map_err(D::Error::custom)?;

        Ok(SerializableDuration(Duration::seconds(
            hours * 3600 + minutes * 60 + seconds,
        )))
    }
}

pub mod date_time {
    use core::fmt;

    use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeZone as _};
    use serde::{
        de::{self, Error, IntoDeserializer, Unexpected, Visitor},
        Deserialize as _, Deserializer,
    };

    pub fn deserialize_local<'de, D>(
        deserializer: D,
    ) -> Result<DateTime<Local>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let naive_datetime = NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S")
            .map_err(Error::custom)?;
        let local_datetime = Local
            .from_local_datetime(&naive_datetime)
            .single()
            .ok_or_else(|| Error::custom("Invalid local datetime"))?;
        Ok(local_datetime)
    }

    pub fn deserialize_local_option<'de, D>(
        deserializer: D,
    ) -> Result<Option<DateTime<Local>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = Option::<String>::deserialize(deserializer)?;
        match s {
            Some(s) => {
                let duration = deserialize_local(s.as_str().into_deserializer())?;
                Ok(Some(duration))
            }
            None => Ok(None),
        }
    }

    pub fn deserialize_yyyymmdd<'de, D>(
        deserializer: D,
    ) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DateVisitor;

        impl<'de> Visitor<'de> for DateVisitor {
            type Value = NaiveDate;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string in the format YYYYMMDD")
            }

            fn visit_str<E>(self, value: &str) -> Result<NaiveDate, E>
            where
                E: de::Error,
            {
                NaiveDate::parse_from_str(value, "%Y%m%d").map_err(|_| {
                    de::Error::invalid_value(Unexpected::Str(value), &self)
                })
            }
        }

        deserializer.deserialize_str(DateVisitor)
    }
}

pub mod duration {
    use chrono::Duration;
    use schemars::gen::SchemaGenerator;
    use schemars::schema::{InstanceType, Schema, SchemaObject};
    use serde::de::{Error as DeError, IntoDeserializer};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let total_seconds = duration.num_seconds();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        let formatted = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);
        serializer.serialize_str(&formatted)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let parts: Vec<&str> = s.split(':').collect();

        if parts.len() != 3 {
            return Err(D::Error::invalid_length(
                parts.len(),
                &"Expected format hh:mm:ss",
            ));
        }

        let hours: i64 = parts[0].parse().map_err(D::Error::custom)?;
        let minutes: i64 = parts[1].parse().map_err(D::Error::custom)?;
        let seconds: i64 = parts[2].parse().map_err(D::Error::custom)?;

        Ok(Duration::hours(hours)
            + Duration::minutes(minutes)
            + Duration::seconds(seconds))
    }

    pub fn serialize_option<S>(
        option_duration: &Option<Duration>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match option_duration {
            Some(d) => serialize(d, serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize_option<'de, D>(
        deserializer: D,
    ) -> Result<Option<Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = Option::<String>::deserialize(deserializer)?;
        match s {
            Some(s) => {
                let duration = deserialize(s.as_str().into_deserializer())?;
                Ok(Some(duration))
            }
            None => Ok(None),
        }
    }

    pub fn schema(_gen: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            format: Some("hh:mm:ss".to_owned()),
            ..Default::default()
        }
        .into()
    }

    pub fn schema_option(_gen: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            format: Some("hh:mm:ss".to_owned()),
            ..Default::default()
        }
        .into()
    }
}
