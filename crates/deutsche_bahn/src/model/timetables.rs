use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};
use serde_with;

use chrono::{DateTime, Local, NaiveDate, ParseError};

use super::{deserialize_path, deserialize_path_opt, timestamp, timestamp_opt};

/// A transport object which keep data for a station
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StationData {
    /// Station name.
    pub name: String, /* name*: string, xml-attribute */

    /// EVA station number.
    pub eva: i64, /* eva*: integer($int64), xml-attribute */

    /// DS100 station code.
    pub ds100: String, /* ds100*: string, xml-attribute */

    /* -- optional fields -- */
    /// List of meta stations.
    /// A sequence of station names separated by the pipe symbols ("|").
    #[serde(alias = "meta")]
    pub meta_stations: Option<String>, /* meta: string, xml-attribute */

    /// List of platforms.
    /// A sequence of platforms separated by the pipe symbols ("|")
    #[serde(alias = "p")]
    pub platforms: Option<String>, /* p: string */

    /* -- not listed in documentation -- */
    pub db: Option<bool>,

    pub creationts: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    #[serde(alias = "1")]
    High, /* "1" */

    #[serde(alias = "2")]
    Medium, /* "2" */

    #[serde(alias = "3")]
    Low, /* "3" */

    #[serde(alias = "4")]
    Done, /* "4" */
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DistributorType {
    #[serde(alias = "s")]
    City, /* "s" */

    #[serde(alias = "r")]
    Region, /* "r" */

    #[serde(alias = "f")]
    LongDistance, /* "f" */

    #[serde(alias = "x")]
    Other, /* "x" */
}

/// An additional message to a given station-based disruption by a specific distributer.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DistributorMessage {
    #[serde(alias = "int")]
    pub internal_text: Option<String>, /* int: string, xml-attribute */

    #[serde(alias = "n")]
    pub distributor_name: Option<String>, /* n: string, xml-attribute */

    #[serde(alias = "t")]
    pub distributor_type: Option<DistributorType>, /* t: string(enum distributorType) */

    /// XML: The time, in ten digit 'YYMMddHHmm' format.
    #[serde(default)]
    #[serde(alias = "ts", with = "timestamp_opt")]
    pub timestamp: Option<DateTime<Local>>, /* ts: string, xml-attribute */
}

/// TODO: rename enum variants to appropriate and descriptive names.
///       Sadly, I could not find any documentation on what those
///       letters mean :/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TripType {
    #[serde(alias = "p")]
    P, /* "p" */

    #[serde(alias = "e")]
    E, /* e */

    #[serde(alias = "z")]
    Z, /* z */

    #[serde(alias = "s")]
    S, /* s */

    #[serde(alias = "h")]
    H, /* h */

    #[serde(alias = "n")]
    N, /* n */
}

/// It's a compound data type that contains common data items that characterize a Trip.
/// The contents is represented as a compact 6-tuple in XML
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TripLabel {
    /// Trip category, e.g. "ICE" or "RE".
    #[serde(alias = "c")]
    pub category: String, /* c*: string, xml-attribute */

    /// Trip/train number, e.g. "4523".
    #[serde(alias = "n")]
    pub trip_or_train_number: String, /* n*: string, xml-attribute */

    /// A unique short-form and only intended to map a trip to specific evu.
    #[serde(alias = "o")]
    pub owner: String, /* o*: string, xml-attribute */

    /* -- optional fields -- */
    #[serde(alias = "f")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_flags: Option<String>, /* f: string, xml-attribute */

    #[serde(alias = "t")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trip_type: Option<TripType>, /* t: string(enum tripType) */
}

/// Message status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// A HIM message (generated through the Hafas Information Manager).
    #[serde(alias = "h")]
    Him, /* "h" */

    /// A message about a quality change.
    #[serde(alias = "q")]
    QualityChange, /* "q" */

    /// A free text message.
    #[serde(alias = "f")]
    Free, /* "f" */

    /// A message about the cause of a delay.
    #[serde(alias = "d")]
    CauseOfDelay, /* "d" */

    /// An IBIS message (generated from IRIS-AP).
    #[serde(alias = "i")]
    Ibis, /* i */

    /// An IBIS message (generated from IRIS-AP) not yet assigned to a train.
    #[serde(alias = "u")]
    UnassignedIbisMessage, /* u */

    /// A major disruption.
    #[serde(alias = "r")]
    Disruption, /* r */

    /// A connection.
    #[serde(alias = "c")]
    Connection, /* c */
}

/// A message that is associated with an event, a stop or a trip
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    /// Message id
    pub id: String, /* id: string, xml-attribute */

    #[serde(alias = "t")]
    pub message_type: MessageType, /* t: string(enum messageType) */

    /// XML: The time, in ten digit 'YYMMddHHmm' format.
    #[serde(alias = "ts", with = "timestamp")]
    pub timestamp: DateTime<Local>, /* ts: string, xml-attribute */

    /* -- optional fields -- */
    #[serde(alias = "c")]
    pub code: Option<String>, /* c: integer, xml-attribute */

    #[serde(alias = "cat")]
    pub category: Option<String>, /* cat: string, xml-attribute */

    #[serde(alias = "del")]
    pub deleted: Option<String>, /* del: integer, xml-attribute */

    #[serde(alias = "dm", default, skip_serializing_if = "Vec::is_empty")]
    pub distributor_messages: Vec<DistributorMessage>, /* dm: [distributorMessage] */

    #[serde(alias = "ec")]
    pub external_category: Option<String>, /* ec: string, xml-attribute */

    /// External link associated with the message.
    #[serde(alias = "elnk")]
    pub external_link: Option<String>, /* elnk: string, xml-attribute */

    #[serde(alias = "ext")]
    pub external_text: Option<String>, /* ext: string, xml-attribute */

    /// XML: The time, in digit 'YYMMddHHmm' format
    #[serde(default)]
    #[serde(alias = "from", with = "timestamp_opt")]
    pub valid_from: Option<DateTime<Local>>, /* from: string, xml-attribute */

    #[serde(alias = "int")]
    pub internal_text: Option<String>, /* int: string, xml-attribute */

    #[serde(alias = "o")]
    pub owner: Option<String>, /* o: string, xml-attribute */

    #[serde(alias = "pr")]
    pub priority: Option<Priority>, /* pr: string(enum priority) */

    #[serde(alias = "tl", default, skip_serializing_if = "Vec::is_empty")]
    pub trip_labels: Vec<TripLabel>, /* tl: [tripLabel] */

    /// XML: The time, in ten digit 'YYMMddHHmm' format.
    #[serde(default)]
    #[serde(alias = "to", with = "timestamp_opt")]
    pub valid_to: Option<DateTime<Local>>, /* to: string, xml-attribute */
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventStatus {
    /// The event was planned.
    /// This status is also used when the cancellation of an event has been revoked.
    #[default]
    #[serde(alias = "p")]
    Planned, /* "p" */

    /// The event was added to the planned data (new stop).
    #[serde(alias = "a")]
    Added, /* "a" */

    /// The event was cancelled (as changedstatus, can apply to planned and added stops)
    #[serde(alias = "c")]
    Cancelled, /* "c" */
}

impl fmt::Display for EventStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Planned => write!(f, "Planned"),
            Self::Added => write!(f, "Added"),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// Not part of the Bahn-API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActualPathStop {
    pub status: EventStatus,
    pub name: String,
}

/// An event (arrival or departure) that is part of a stop
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    /* -- properties not part of bahn-api -- */
    #[serde(default)]
    pub actual_path: Vec<ActualPathStop>,

    #[serde(default)]
    pub actual_status: EventStatus,

    /* -- optional fields -- */
    #[serde(alias = "cde")]
    pub changed_distant_endpoint: Option<String>, /* cde: string, xml-attribute */

    /// Time when the cancellation of this stop was created.
    /// XML: The time, in ten digit 'YYMMddHHmm' format.
    #[serde(alias = "clt", with = "timestamp_opt", default)]
    pub cancellation_time: Option<DateTime<Local>>, /* clt: string, xml-attribute */

    #[serde(alias = "cp")]
    pub changed_platform: Option<String>, /* cp: string, xml-attribute */

    #[serde(alias = "cpth", deserialize_with = "deserialize_path_opt", default)]
    #[serde(skip_serializing)] // clients should use 'actual_path'
    pub changed_path: Option<Vec<String>>, /* cpth: string, xml-attribute */

    /// TODO: 'changed_status' is a guess. There is no description in the documentation.
    #[serde(alias = "cs")]
    pub changed_status: Option<EventStatus>, /* cs: eventStatus */

    /// New estimated or actual departure or arrival time.
    /// XML: The time, in ten digit 'YYMMddHHmm' format.
    #[serde(alias = "ct", with = "timestamp_opt", default)]
    pub changed_time: Option<DateTime<Local>>, /* ct: string, xml-attribute */

    #[serde(alias = "dc")]
    pub distant_change: Option<String>, /* dc: integer, xml-attribute */

    /// 1 if the event should not be shown on WBT because travellers are not supposed
    /// to enter or exit the train at this stop.
    #[serde(alias = "hi")]
    pub hidden: Option<i64>, /* hi: integer, xml-attribute */

    /// The line indicator (e.g. "3" for an S-Bahn or "45S" for a bus).
    #[serde(alias = "l")]
    pub line: Option<String>, /* l: string, xml-attribute */

    #[serde(alias = "m", default, skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<Message>, /* m: [message] */

    #[serde(alias = "pde")]
    pub planned_distant_endpoint: Option<String>, /* pde: string, xml-attribute */

    #[serde(alias = "pp")]
    pub planned_platform: Option<String>, /* pp: string, xml-attribute */

    /// A sequence of station names separated by the pipe symbols ('|').
    /// E.g.: 'Mainz Hbf|Rüsselsheim|Frankfurt(M) Flughafen'.
    ///
    /// For arrival, the path indicates the stations that
    /// come before the current station. The first element gehn is the trip's
    /// start station.
    ///
    /// For departure, the path indicates the stations that come after the current station.
    /// The last element in the path then is the trip's destination station.
    ///
    /// Note that the current station is never included in the path
    /// (neither for arrival nor for departure).
    #[serde(alias = "ppth", deserialize_with = "deserialize_path", default)]
    #[serde(skip_serializing)] // clients should use 'actual_path'
    pub planned_path: Vec<String>, /* ppth: string, xml-attribute */

    // TODO: 'planned_status' is a guess. There is no description in the documentation.
    #[serde(alias = "ps")]
    pub planned_status: Option<EventStatus>, /* ps: eventStatus */

    /// Planned departure or arrival time.
    /// XML: The time, in then digit 'YYMMddHHmm' format.
    #[serde(default)]
    #[serde(alias = "pt", with = "timestamp_opt")]
    pub planned_time: Option<DateTime<Local>>, /* pt: string, xml-attribute */

    /// Trip id of the next or previous train of a shared train.
    /// At the start stop this references the previous trip, at the last stop
    /// it references the next trip.
    /// E.g. '2016448009055686515-1403311438-1'
    #[serde(alias = "tra")]
    pub transition: Option<String>, /* tra: string */

    /// A sequence of trip id separated by the pipe symbols (|).
    pub wings: Option<String>, /* wings: string, xml-attribute */
}

impl Event {
    pub fn calculate_all(&mut self) {
        self.calculate_actual_status();
        self.calculate_actual_path();
    }

    pub fn calculate_actual_status(&mut self) {
        self.actual_status = self
            .changed_status
            .clone()
            .unwrap_or(self.planned_status.clone().unwrap_or(EventStatus::Planned));
    }

    pub fn calculate_actual_path(&mut self) {
        self.actual_path.clear();
        /* if path has changed... */
        if let Some(changed_path) = &self.changed_path {
            let (mut i, mut j) = (0usize, 0usize);
            while i < changed_path.len() || j < self.planned_path.len() {
                /* last stop(s) cancelled */
                if i >= changed_path.len() {
                    self.actual_path.push(ActualPathStop {
                        status: EventStatus::Cancelled,
                        name: self.planned_path[j].clone(),
                    });
                    j += 1;
                    continue;
                }
                /* last stop(s) added */
                if j >= self.planned_path.len() {
                    self.actual_path.push(ActualPathStop {
                        status: EventStatus::Added,
                        name: changed_path[i].clone(),
                    });
                    i += 1;
                    continue;
                }
                /* planned stop */
                if changed_path[i] == self.planned_path[j] {
                    self.actual_path.push(ActualPathStop {
                        status: EventStatus::Planned,
                        name: changed_path[i].clone(),
                    });
                    i += 1;
                    j += 1;
                    continue;
                }

                /* stops added and cancelled */
                let mut changed = false;

                for pk in j..self.planned_path.len() + 1 {
                    /* added stop(s) */
                    for mut ck in i..changed_path.len() {
                        /* difference, continue search for common path stop */
                        if pk != self.planned_path.len()
                            && changed_path[ck] != self.planned_path[pk]
                        {
                            continue;
                        }
                        /* special case where there are no more common stops in
                        planned path and changed path */
                        if pk == self.planned_path.len() {
                            /* set ck value so all remaining stops are added */
                            ck = changed_path.len();
                        }
                        /* append cancelled stops */
                        self.actual_path.append(
                            &mut self.planned_path[j..pk]
                                .iter()
                                .map(|elem| ActualPathStop {
                                    status: EventStatus::Cancelled,
                                    name: elem.clone(),
                                })
                                .collect(),
                        );
                        /* append new stops */
                        self.actual_path.append(
                            &mut changed_path[i..ck]
                                .iter()
                                .map(|elem| ActualPathStop {
                                    status: EventStatus::Added,
                                    name: elem.clone(),
                                })
                                .collect(),
                        );
                        /* set to new common start */
                        i = ck;
                        j = pk;
                        changed = true;
                        break;
                    }
                    /* break outer loop upon change */
                    if changed {
                        break;
                    }
                }
            }

            /* ...if path has NOT changed */
        } else {
            self.actual_path.append(
                &mut self
                    .planned_path
                    .iter()
                    .map(|elem| ActualPathStop {
                        status: EventStatus::Planned,
                        name: elem.clone(),
                    })
                    .collect(),
            )
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionStatus {
    /// This (regular) connection is waiting.
    #[serde(alias = "w")]
    Waiting, /* "w" */

    /// This (regular) connection CANNOT wait.
    #[serde(alias = "n")]
    Transition, /* "n" */

    /// This is an alternative (unplanned) connection that has been introduced
    /// as a replecement for one regular connection that cannot wait.
    /// The connections "tl" (triplabel) attribute might in this case refer to
    /// the replaced connection (or more specifically the trip from that connection).
    /// Alternative connections are always waiting (they are removed otherwise).
    #[serde(alias = "a")]
    Alternative, /* "a" */
}

/// It's information about a connected train at a particular stop.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    #[serde(alias = "cs")]
    pub connection_status: ConnectionStatus, /* cs*: string(enum connectionStatus) */

    id: String, /* id*: string, xml-attribute */

    #[serde(alias = "s")]
    pub timetable_stop: TimetableStop, /* s*: timetableStop */

    /// XML: The time, in ten digit 'YYMMddHHmm' format.
    #[serde(alias = "ts", with = "timestamp")]
    pub timestamp: DateTime<Local>, /* ts*: string, xml-attribute */

    /* -- optional fields -- */
    /// EVA station number.
    pub eva: Option<i64>, /* eva: integer($int64), xml-attribute */

    #[serde(alias = "ref")]
    pub ref_timetable_stop: Option<TimetableStop>, /* ref: timetableStop */
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DelaySource {
    /// LeiBit/LeiDis.
    #[serde(alias = "L")]
    Leibit, /* "L" */

    /// RISNE AUT IRIS-NE (automatisch).
    #[serde(alias = "NA")]
    RisneAutIrisNe, /* "NA" */

    /// RISNE MAN IRIS-NE (manuell).
    #[serde(alias = "NM")]
    RisneManIrisNe, /* "NM" */

    /// VDV Prognosen durch dritte EVU über VDVin.
    #[serde(alias = "V")]
    Vdv, /* "V" */

    /// ISTP automatisch.
    #[serde(alias = "IA")]
    IstpAut, /* "IA" */

    /// ISTP manuell.
    #[serde(alias = "IM")]
    IstpMan, /* IM */

    /// Automatische Prognose durch Prognoseautomat.
    #[serde(alias = "A")]
    AutomaticPrognosis, /* "A" */
}

/// It's the history of all delay-messages for a stop.
/// This element extends HistoricChange
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoricDelay {
    /* -- optinal fields -- */
    /// The arrival event.
    /// XML: The time, in ten digit 'YYMMddHHmm' format.
    #[serde(alias = "ar", with = "timestamp_opt", default)]
    pub arrival: Option<DateTime<Local>>, /* ar: string, xml-attribute */

    /// The departure event.
    /// XML: The time, in ten digit 'YYMMddHHmm' format.
    #[serde(alias = "dp", with = "timestamp_opt", default)]
    pub departure: Option<DateTime<Local>>, /* dp: string, xml-attribute */

    /// A detailed description of delay cause.
    #[serde(alias = "cod")]
    pub cause_of_delay: Option<String>, /* cod: string, xml-attribute */

    #[serde(alias = "src")]
    pub delay_source: Option<DelaySource>, /* src: string(enum delaySource) */

    /// XML: The time, in ten digit 'YYMMddHHmm' format.
    #[serde(default)]
    #[serde(alias = "ts", with = "timestamp_opt")]
    pub timestamp: Option<DateTime<Local>>, /* ts: string, xml-attribute */
}

/// It's the history of all platform-changes for a stop.
/// This element extends HistoricChange.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoricPlatformChange {
    /* -- optional fields -- */
    #[serde(alias = "ar")]
    pub arrival_platform: Option<String>, /* ar: string, xml-attribute */

    #[serde(alias = "dp")]
    pub departure_platform: Option<String>, /* dp: string, cml-attribute */

    /// Detailed cause of track change.
    #[serde(alias = "cot")]
    pub cause_of_track_change: Option<String>, /* cot: string, xml-attribute */

    /// XML: The time, in ten digit 'YYMMddHHmm' format.
    #[serde(default)]
    #[serde(alias = "ts", with = "timestamp_opt")]
    pub timestamp: Option<DateTime<Local>>, /* ts: string, xml-attribute */
}

/// It's a reference to another trip, which holds its label and reference trips, if available.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TripReference {
    #[serde(alias = "tl")]
    pub triplabel: TripLabel, /* tl: tripLabel */

    /* -- optional fileds -- */
    #[serde(alias = "rt", default, skip_serializing_if = "Vec::is_empty")]
    pub referred_trips: Vec<TripLabel>, /* rt: [tripLabel] */
}

/// The reference trips relation to the stop, which contains it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReferenceTripRelationToStop {
    /// The reference trip ends before that stop.
    #[serde(alias = "b")]
    Before, /* "b" */

    /// The reference trip ends at that stop.
    #[serde(alias = "e")]
    End, /* "e" */

    /// The stop is between reference trips start and end,
    /// in other words, the stop is contained within its travel path.
    #[serde(alias = "c")]
    Between, /* "c" */

    /// The reference trip starts at that stop.
    #[serde(alias = "s")]
    Start, /* "s" */

    /// The reference trip starts after that stop.
    #[serde(alias = "a")]
    After, /* "a" */
}

/// It's a compound data type that contains common data items that charactarize
/// a reference trip stop.
/// The contents is represented as a compact 4-tuple in XML.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceTripStopLabel {
    /// The EVA number of the correspondent stop of the regular trip.
    pub eva: i64, /* eva*: integer($int64), xml-attribute */

    /// The index of the correspondent stop of the regular trip.
    #[serde(alias = "i")]
    pub index: i64, /* i*: integer, xml-attribute */

    /// The (long) name of the correspondent stop of the regular trip.
    #[serde(alias = "n")]
    pub name: String, /* n*: sstring, xml-attribute */

    /// The planned time of the correspondent stop of the regular trip.
    /// TODO: no format specified in documentation. Probably 'YYMMddHHmm'...
    #[serde(alias = "pt")]
    pub planned_time: String, /* pt: string, xml-attribute */
}

/// It's a compund data type that contains common data items that characterize
/// a reference trip.
/// The contents is represented as a compact 3-tuple in XML.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceTripLabel {
    /// Trip category, e.g. "ICE" or "RE".
    /// TODO: maybe introduce a 'TripCategory' enum?
    #[serde(alias = "c")]
    pub category: String, /* c*: string, xml-attribute */

    /// Trip/train number, e.g. "4523".
    #[serde(alias = "n")]
    pub trip_or_train_number: String, /* n*: string, xml-attribute */
}

/// A reference trip is another real trip, but it doesn't have its own stops and events.
/// It refers only to its referenced regular trip. The reference trip collects mainly all
/// different attributes of the referenced regular trip.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceTrip {
    /// The cancellation flag. True means, the reference trip is cancelled.
    #[serde(alias = "c")]
    pub cancellation_flag: bool, /* c*: boolean, xml-attribute */

    /// TODO: better name! Meaning is not stated in the documentation.
    pub ea: ReferenceTripStopLabel, /* ea*: referenceTripStopLabel */

    /// An id that uniquely identifies the reference trip.
    /// It consists of the following two elements separated by dashes:
    ///     - A 'daily trip id' that uniquely identifies a reference trip within one day.
    ///       This id is typically reused on subsequent days. This could be negative.
    ///     - A 10-digit data specifier (YYMMddHHmm) that indicates the planned departure
    ///       date of the referenced regular trip from its start station.
    ///
    /// EXAMPLE
    /// '-7874571842864554321-1403311221' would be used for a trip with
    /// daily trip id '-7874571842864554321' that starts on march the 31th 2014.
    pub id: String, /* id*: string, xml-attribute */

    #[serde(alias = "rtl")]
    pub reference_trip_label: ReferenceTripLabel, /* rtl*: referenceTripLabel */

    #[serde(alias = "sd")]
    pub reference_trip_stop_label: ReferenceTripStopLabel, /* sd*: referenceTripStopLabel */
}

/// A reference trip realtion holds how a reference trip is related to a stop,
/// for instance the reference trip starts after the stop.
/// Stop contains a collection of that type, only if reference trips are available.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceTripRelation {
    #[serde(alias = "rt")]
    pub reference_trip: ReferenceTrip, /* rt*: referenceTrip */

    #[serde(alias = "rts")]
    pub reference_trip_relation_to_stop: ReferenceTripRelationToStop, /* rts*: string(enum referenceTripRelationToStop) */
}

/// An id that uniquely identifies the stop.
/// It consists of the following three elements separated by dashes
///     - A 'daily trip id' that uniquely identifies a trip within one day.
///       This id is typically reused on subsequent days. This could be negative.
///     - A 6-digit date specifier (YYMMdd) that indicates the planned departure
///       date of the trip from its start station.
///     - An index ('index_of_stop_in_trip') that indicates the position of the stop within the trip
///       (in rare cases, one trip may arrive multiple times at one station).
///       Added trips get indices above 100.
///
/// EXAMPLE
/// '-7874571842864554321-1403311221-11' would be used for a trip with
/// daily trip id '-7874571842864554321' that starts on march the 31th 2014
/// and where the current station is the 11th stop.
#[derive(Clone, Debug)]
pub struct TimetableStopId {
    pub daily_trip_id: String,
    pub date_specifier: String,
    pub index_of_stop_in_trip: i32,
}

impl TimetableStopId {
    pub fn full_id_string(&self) -> String {
        format!(
            "{}-{}-{}",
            &self.daily_trip_id, &self.date_specifier, &self.index_of_stop_in_trip
        )
    }

    pub fn trip_id_string(&self) -> String {
        format!("{}-{}", &self.daily_trip_id, &self.date_specifier)
    }

    pub fn date(&self) -> Result<NaiveDate, ParseError> {
        NaiveDate::parse_from_str(&self.date_specifier.as_str()[..6], "%y%m%d")
    }

    pub fn parse_str(from: &str) -> Result<Self, String> {
        if from.len() < 5 {
            return Err("id too short!".to_owned());
        }

        let mut parts = vec![];
        let mut current = String::new();
        for c in from.chars() {
            if c == '-' && !current.is_empty() {
                parts.push(current.clone());
                current.clear();
            } else {
                current.push(c);
            }
        }
        if !current.is_empty() {
            parts.push(current);
        }

        if parts.len() != 3 {
            return Err(format!(
                "id should consist of 3 parts. found {}",
                parts.len()
            ));
        }

        if let Ok(stop_index) = parts[2].parse::<i32>() {
            Ok(Self {
                daily_trip_id: parts[0].clone(),
                date_specifier: parts[1].clone(),
                index_of_stop_in_trip: stop_index,
            })
        } else {
            Err(format!("stop-index is not a valid i32: {}", parts[2]))
        }
    }
}

impl Serialize for TimetableStopId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.full_id_string().as_str())
    }
}

struct TimetableStopIdVisitor;

impl<'de> serde::de::Visitor<'de> for TimetableStopIdVisitor {
    type Value = TimetableStopId;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string representing a timetable-stop-id.")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        TimetableStopId::parse_str(value).map_err(|err| {
            E::custom(format!("not a valid timetable stop id: {value} -> {err}"))
        })
    }
}

impl<'de> Deserialize<'de> for TimetableStopId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(TimetableStopIdVisitor)
    }
}

/// A stop is a part of a Timetable
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimetableStop {
    /// An id that uniquely identifies the stop.
    /// It consists of the following three elements separated by dashes
    ///     - A 'daily trip id' that uniquely identifies a trip within one day.
    ///       This id is typically reused on subsequent days. This could be negative.
    ///     - A 6-digit date specifier (YYMMdd) that indicates the planned departure
    ///       date of the trip from its start station.
    ///     - An index that indicates the position of the stop within the trip
    ///       (in rare cases, one trip may arrive multiple times at one station).
    ///       Added trips get indices above 100.
    ///
    /// EXAMPLE
    /// '-7874571842864554321-1403311221-11' would be used for a trip with
    /// daily trip id '-7874571842864554321' that starts on march the 31th 2014
    /// and where the current station is the 11th stop.
    pub id: TimetableStopId, /* id*: string, attribute; TODO: create own datatype? */

    /// The eva code of the station of this stop.
    /// Option is intentional, as the db breaks the api here at GET /plan
    pub eva: Option<i64>, /* eva*: integer($int64), attribute */

    /* -- optional fields -- */
    #[serde(alias = "ar")]
    pub arrival: Option<Event>, /* ar: event */

    #[serde(alias = "dp")]
    pub departure: Option<Event>, /* dp: event */

    #[serde(alias = "conn", default, skip_serializing_if = "Vec::is_empty")]
    pub connections: Vec<Connection>, /* conn: [connection] */

    #[serde(alias = "hd", default, skip_serializing_if = "Vec::is_empty")]
    pub historic_delays: Vec<HistoricDelay>, /* hd: [historicDelay] */

    #[serde(alias = "hpc", default, skip_serializing_if = "Vec::is_empty")]
    pub historic_platform_changes: Vec<HistoricPlatformChange>, /* hpc: [historicPlatformChange] */

    #[serde(alias = "m", default, skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<Message>, /* m: [message] */

    #[serde(alias = "ref")]
    pub trip_reference: Option<TripReference>, /* ref: tripReference */

    #[serde(alias = "rtr", default, skip_serializing_if = "Vec::is_empty")]
    pub reference_trip_relations: Vec<ReferenceTripRelation>, /* rtr: [referenceTripRelation] */

    #[serde(alias = "tl")]
    pub trip_label: Option<TripLabel>, /* tl: tripLabel */
}

impl TimetableStop {
    /// Calculate all values that are not part of the Bahn-API
    pub fn calculate_all(&mut self) {
        match &mut self.arrival {
            Some(event) => event.calculate_all(),
            _ => {}
        }
        match &mut self.departure {
            Some(event) => event.calculate_all(),
            _ => {}
        }
    }

    pub fn create_full_path_without_own_stop(&self) -> Vec<ActualPathStop> {
        let mut result = self.arrival_path().map_or(Vec::new(), |ar| ar.clone());
        if let Some(dp) = self.departure_path() {
            result.append(&mut dp.clone());
        }
        result
    }

    pub fn arrival_path(&self) -> Option<&Vec<ActualPathStop>> {
        self.arrival.as_ref().map(|event| &event.actual_path)
    }

    pub fn departure_path(&self) -> Option<&Vec<ActualPathStop>> {
        self.departure.as_ref().map(|event| &event.actual_path)
    }

    pub fn status(&self) -> EventStatus {
        let arrival_status = self
            .arrival
            .as_ref()
            .map_or(EventStatus::Planned, |event| event.actual_status.clone());
        let departure_status = self
            .departure
            .as_ref()
            .map_or(EventStatus::Planned, |event| event.actual_status.clone());

        match (arrival_status, departure_status) {
            (EventStatus::Cancelled, _) => EventStatus::Cancelled,
            (_, EventStatus::Cancelled) => EventStatus::Cancelled,
            (EventStatus::Added, _) => EventStatus::Added,
            (_, EventStatus::Added) => EventStatus::Added,
            _ => EventStatus::Planned,
        }
    }

    pub fn is_added(&self) -> bool {
        matches!(
            self.arrival
                .as_ref()
                .and_then(|a| a.planned_status.clone())
                .or(self
                    .departure
                    .as_ref()
                    .and_then(|d| d.planned_status.clone())),
            Some(EventStatus::Added),
        )
    }
}

/// A timetable is made of a set of TimetableStops and a potential Disruption
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Timetable {
    /* -- optional fields -- */
    /// Eva station number.
    pub eva: Option<i64>, /* eva: integer($int64), xml-attribute */

    #[serde(alias = "m", default, skip_serializing_if = "Vec::is_empty")]
    pub messages: Vec<Message>, /* m: [message] */

    /// Station name.
    #[serde(alias = "station")]
    pub station_name: Option<String>, /* station: string, xml-attribute */

    //#[serde(alias = "s", default, skip_serializing_if = "Vec::is_empty")]
    #[serde(alias = "s", default)]
    pub stops: Vec<TimetableStop>, /* s: [timetableStop] */

    /* -- fields not provided by the Bahn-API */
    #[serde(default)]
    #[serde(with = "timestamp_opt")]
    pub live_data_last_updated_at: Option<DateTime<Local>>,
}
