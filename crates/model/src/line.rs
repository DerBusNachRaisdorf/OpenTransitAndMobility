use std::cmp;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utility::{
    edit_distance::edit_distance,
    id::{HasId, Id},
    math::sigmoid,
};

use crate::{agency::Agency, ExampleData, Mergable, Subject};

/// taken from gtfs.
#[serde_with::skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum LineType {
    TramStreetcarOrLighrail,
    SubwayOrMetro,
    Rail,
    Bus,
    Ferry,
    CableTram,
    AerialLiftOrSuspendedCableCar,
    Funicular,
    Trolleybus,
    Monorail,
}

struct LineTypeSimilarityVec {
    tram_streetcar_or_lightrail: f64,
    subway_or_metro: f64,
    rail: f64,
    bus: f64,
    ferry: f64,
    cable_tram: f64,
    aerial_lift_or_suspended_cable_car: f64,
    funicular: f64,
    trolleybus: f64,
    monorail: f64,
}

impl LineType {
    pub fn similarity(&self, other: &Self) -> f64 {
        let similarity_vector = self.similarity_vec();

        match other {
            LineType::TramStreetcarOrLighrail => {
                similarity_vector.tram_streetcar_or_lightrail
            }
            LineType::SubwayOrMetro => similarity_vector.subway_or_metro,
            LineType::Rail => similarity_vector.rail,
            LineType::Bus => similarity_vector.bus,
            LineType::Ferry => similarity_vector.ferry,
            LineType::CableTram => similarity_vector.cable_tram,
            LineType::AerialLiftOrSuspendedCableCar => {
                similarity_vector.aerial_lift_or_suspended_cable_car
            }
            LineType::Funicular => similarity_vector.funicular,
            LineType::Trolleybus => similarity_vector.trolleybus,
            LineType::Monorail => similarity_vector.monorail,
        }
    }

    fn similarity_vec(&self) -> LineTypeSimilarityVec {
        match self {
            LineType::TramStreetcarOrLighrail => LineTypeSimilarityVec {
                tram_streetcar_or_lightrail: 1.0,
                subway_or_metro: 0.6,
                rail: 0.8,
                bus: 0.5,
                ferry: 0.2,
                cable_tram: 0.7,
                aerial_lift_or_suspended_cable_car: 0.4,
                funicular: 0.5,
                trolleybus: 0.5,
                monorail: 0.6,
            },
            LineType::SubwayOrMetro => LineTypeSimilarityVec {
                tram_streetcar_or_lightrail: 0.6,
                subway_or_metro: 1.0,
                rail: 0.7,
                bus: 0.4,
                ferry: 0.2,
                cable_tram: 0.5,
                aerial_lift_or_suspended_cable_car: 0.3,
                funicular: 0.4,
                trolleybus: 0.3,
                monorail: 0.5,
            },
            LineType::Rail => LineTypeSimilarityVec {
                tram_streetcar_or_lightrail: 0.8,
                subway_or_metro: 0.7,
                rail: 1.0,
                bus: 0.3,
                ferry: 0.1,
                cable_tram: 0.5,
                aerial_lift_or_suspended_cable_car: 0.3,
                funicular: 0.4,
                trolleybus: 0.3,
                monorail: 0.4,
            },
            LineType::Bus => LineTypeSimilarityVec {
                tram_streetcar_or_lightrail: 0.5,
                subway_or_metro: 0.4,
                rail: 0.3,
                bus: 1.0,
                ferry: 0.3,
                cable_tram: 0.2,
                aerial_lift_or_suspended_cable_car: 0.1,
                funicular: 0.2,
                trolleybus: 0.4,
                monorail: 0.3,
            },
            LineType::Ferry => LineTypeSimilarityVec {
                tram_streetcar_or_lightrail: 0.2,
                subway_or_metro: 0.2,
                rail: 0.1,
                bus: 0.3,
                ferry: 1.0,
                cable_tram: 0.1,
                aerial_lift_or_suspended_cable_car: 0.2,
                funicular: 0.1,
                trolleybus: 0.1,
                monorail: 0.2,
            },
            LineType::CableTram => LineTypeSimilarityVec {
                tram_streetcar_or_lightrail: 0.7,
                subway_or_metro: 0.5,
                rail: 0.5,
                bus: 0.2,
                ferry: 0.1,
                cable_tram: 1.0,
                aerial_lift_or_suspended_cable_car: 0.6,
                funicular: 0.5,
                trolleybus: 0.3,
                monorail: 0.4,
            },
            LineType::AerialLiftOrSuspendedCableCar => LineTypeSimilarityVec {
                tram_streetcar_or_lightrail: 0.4,
                subway_or_metro: 0.3,
                rail: 0.3,
                bus: 0.1,
                ferry: 0.2,
                cable_tram: 0.6,
                aerial_lift_or_suspended_cable_car: 1.0,
                funicular: 0.3,
                trolleybus: 0.2,
                monorail: 0.3,
            },
            LineType::Funicular => LineTypeSimilarityVec {
                tram_streetcar_or_lightrail: 0.5,
                subway_or_metro: 0.4,
                rail: 0.4,
                bus: 0.2,
                ferry: 0.1,
                cable_tram: 0.5,
                aerial_lift_or_suspended_cable_car: 0.3,
                funicular: 1.0,
                trolleybus: 0.2,
                monorail: 0.3,
            },
            LineType::Trolleybus => LineTypeSimilarityVec {
                tram_streetcar_or_lightrail: 0.5,
                subway_or_metro: 0.3,
                rail: 0.3,
                bus: 0.4,
                ferry: 0.1,
                cable_tram: 0.3,
                aerial_lift_or_suspended_cable_car: 0.2,
                funicular: 0.2,
                trolleybus: 1.0,
                monorail: 0.3,
            },
            LineType::Monorail => LineTypeSimilarityVec {
                tram_streetcar_or_lightrail: 0.6,
                subway_or_metro: 0.5,
                rail: 0.4,
                bus: 0.3,
                ferry: 0.2,
                cable_tram: 0.4,
                aerial_lift_or_suspended_cable_car: 0.3,
                funicular: 0.3,
                trolleybus: 0.3,
                monorail: 1.0,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Line {
    pub name: Option<String>,
    pub kind: LineType,
    #[serde(skip)]
    pub agency_id: Option<Id<Agency>>,
}

impl Mergable for Line {
    fn merge(self, other: Self) -> Self {
        Line {
            name: other.name.or(self.name),
            kind: other.kind,
            agency_id: other.agency_id.or(self.agency_id),
        }
    }
}

impl Subject for Line {
    fn same_subject_as(&self, other: &Self) -> Option<f64> {
        const NAME_WEIGHT: f64 = 0.5;
        const AGENCY_WEIGHT: f64 = 0.3;
        const KIND_WEIGHT: f64 = 0.2;

        // agency similarty (binary, exclusion criteria if present for both)
        let agency_similarity = match (&self.agency_id, &other.agency_id) {
            (Some(a), Some(b)) if a != b => return None,
            (Some(a), Some(b)) if a == b => 1.0,
            _ => 0.0,
        };

        // line type similarity
        let type_similarity = self.kind.similarity(&other.kind);

        // name similarity (must be present for both)
        let Some(names) =
            self.name.as_ref().zip(other.name.as_ref()).map(|(a, b)| {
                [a, b].map(|name| {
                    name.to_lowercase()
                        .chars()
                        .filter(|c| c.is_alphanumeric())
                        .collect::<String>()
                })
            })
        else {
            return None;
        };
        let name_similarity = edit_distance(&names[0], &names[1]) as f64
            / cmp::max(names[0].len(), names[1].len()) as f64;

        let compound = NAME_WEIGHT * name_similarity
            + AGENCY_WEIGHT * agency_similarity
            + KIND_WEIGHT * type_similarity;

        Some(sigmoid(compound))
    }
}

impl HasId for Line {
    type IdType = String;
}

impl ExampleData for Line {
    fn example_data() -> Self {
        Self {
            name: Some("erx RE83".to_owned()),
            kind: LineType::Rail,
            agency_id: Some(Id::new("erixx-holstein".to_owned())),
        }
    }
}
