use serde::{Deserialize, Serialize};

use crate::core::read_models::protective_measures::{
    ProtectiveMeasureWithExtensions, ProtectiveMeasureWithExtensionsAndEntities,
};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProtectiveMeasureResponse {
    Simple(ProtectiveMeasureWithExtensions),
    WithEntities(ProtectiveMeasureWithExtensionsAndEntities),
}
