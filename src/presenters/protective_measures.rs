use std::sync::Arc;

use log::error;
use serde::{Deserialize, Serialize};

use crate::core::application_error::ApplicationError as AppError;
use crate::core::contracts::repository::cities::CityRepository;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::extensions::ExtensionRepository;
use crate::core::contracts::repository::offenders::OffenderReadRepository;
use crate::core::contracts::repository::victims::VictimReadRepository;
use crate::core::entities::protective_measures::{ProtectiveMeasure, ProtectiveMeasureExtension};
use crate::core::pagination::PaginatedResult;
use crate::core::read_models::protective_measures::{
    ProtectiveMeasureWithExtensions, ProtectiveMeasureWithRelations,
};
use crate::utils::pagination::Pagination;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProtectiveMeasureResponse {
    Simple(ProtectiveMeasureWithExtensions),
    WithEntities(ProtectiveMeasureWithRelations),
}

pub struct ProtectiveMeasurePresenter {
    extension_repository: Arc<dyn ExtensionRepository>,
    victim_read_repository: Arc<dyn VictimReadRepository>,
    offender_read_repository: Arc<dyn OffenderReadRepository>,
    city_repository: Arc<dyn CityRepository>,
}

impl ProtectiveMeasurePresenter {
    pub fn new(
        extension_repository: Arc<dyn ExtensionRepository>,
        victim_read_repository: Arc<dyn VictimReadRepository>,
        offender_read_repository: Arc<dyn OffenderReadRepository>,
        city_repository: Arc<dyn CityRepository>,
    ) -> Self {
        Self {
            extension_repository,
            victim_read_repository,
            offender_read_repository,
            city_repository,
        }
    }

    pub async fn build_response(
        &self,
        measure: ProtectiveMeasure,
        include_related_entities: bool,
    ) -> Result<ProtectiveMeasureResponse, AppError> {
        let extensions = self
            .extension_repository
            .get_extensions_by_measure(measure.id)
            .await
            .map_err(|e| {
                error!(
                    "[ProtectiveMeasurePresenter] Error fetching extensions: {:?}",
                    e
                );
                AppError::InternalServerError
            })?;

        if include_related_entities {
            Ok(ProtectiveMeasureResponse::WithEntities(
                self.build_with_entities(measure, extensions).await?,
            ))
        } else {
            Ok(ProtectiveMeasureResponse::Simple(
                ProtectiveMeasureWithExtensions {
                    measure,
                    extensions,
                },
            ))
        }
    }

    pub async fn build_responses(
        &self,
        measures: Vec<ProtectiveMeasure>,
        include_related_entities: bool,
        pagination: Pagination,
        total_items: i64,
    ) -> Result<PaginatedResult<ProtectiveMeasureResponse>, AppError> {
        let mut items = Vec::with_capacity(measures.len());
        for measure in measures {
            items.push(
                self.build_response(measure, include_related_entities)
                    .await?,
            );
        }
        Ok(PaginatedResult {
            items,
            page: pagination.page,
            page_size: pagination.page_size,
            total_items,
        })
    }

    async fn build_with_entities(
        &self,
        measure: ProtectiveMeasure,
        extensions: Vec<ProtectiveMeasureExtension>,
    ) -> Result<ProtectiveMeasureWithRelations, AppError> {
        let victim = self
            .victim_read_repository
            .get_victim_by_id(measure.victim_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => {
                    AppError::NotFound(format!("Victim with id '{}' not found", measure.victim_id))
                }
                _ => {
                    error!(
                        "[ProtectiveMeasurePresenter] Error fetching victim: {:?}",
                        e
                    );
                    AppError::InternalServerError
                }
            })?;

        let offender = self
            .offender_read_repository
            .get_offender_by_id(measure.offender_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => AppError::NotFound(format!(
                    "Offender with id '{}' not found",
                    measure.offender_id
                )),
                _ => {
                    error!(
                        "[ProtectiveMeasurePresenter] Error fetching offender: {:?}",
                        e
                    );
                    AppError::InternalServerError
                }
            })?;

        let court_district = self
            .city_repository
            .get_city_by_id(measure.court_district_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => AppError::NotFound(format!(
                    "City with id '{}' not found",
                    measure.court_district_id
                )),
                _ => {
                    error!(
                        "[ProtectiveMeasurePresenter] Error fetching court district: {:?}",
                        e
                    );
                    AppError::InternalServerError
                }
            })?;

        Ok(ProtectiveMeasureWithRelations {
            measure,
            extensions,
            victim: victim.into(),
            offender: offender.into(),
            court_district: court_district.into(),
        })
    }
}
