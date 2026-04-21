use serde::{Deserialize, Serialize};

use crate::core::entities::protective_measures::ProtectiveMeasure;
use crate::core::read_models::cities::CitySummary;
use crate::core::read_models::offenders::OffenderSummary;
use crate::core::read_models::victims::VictimSummary;

#[derive(Serialize, Deserialize, Debug)]
pub struct ProtectiveMeasureSimple {
    #[serde(flatten)]
    pub measure: ProtectiveMeasure,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProtectiveMeasureWithRelations {
    #[serde(flatten)]
    pub measure: ProtectiveMeasure,
    pub victim: VictimSummary,
    pub offender: OffenderSummary,
    pub court_district: CitySummary,
}
