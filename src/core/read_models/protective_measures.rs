use serde::{Deserialize, Serialize};

use crate::core::entities::protective_measures::{ProtectiveMeasure, ProtectiveMeasureExtension};
use crate::core::read_models::cities::CityComplement;
use crate::core::read_models::offenders::OffenderComplement;
use crate::core::read_models::victims::VictimComplement;

#[derive(Serialize, Deserialize, Debug)]
pub struct ProtectiveMeasureWithExtensions {
    #[serde(flatten)]
    pub measure: ProtectiveMeasure,
    pub extensions: Vec<ProtectiveMeasureExtension>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProtectiveMeasureWithExtensionsAndEntities {
    #[serde(flatten)]
    pub measure: ProtectiveMeasure,
    pub extensions: Vec<ProtectiveMeasureExtension>,
    pub victim: VictimComplement,
    pub offender: OffenderComplement,
    pub court_district: CityComplement,
}
