use std::sync::Arc;

use log::error;
use serde::{Deserialize, Serialize};

use crate::core::application_error::ApplicationError as AppError;
use crate::core::contracts::repository::cities::CityRepository;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::contracts::repository::offenders::OffenderReadRepository;
use crate::core::contracts::repository::victims::VictimReadRepository;
use crate::core::entities::protective_measures::ProtectiveMeasure;
use crate::core::pagination::PaginatedResult;
use crate::core::read_models::protective_measures::{
    ProtectiveMeasureSimple, ProtectiveMeasureWithRelations,
};
use crate::utils::pagination::Pagination;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProtectiveMeasureResponse {
    Simple(ProtectiveMeasureSimple),
    WithEntities(ProtectiveMeasureWithRelations),
}

pub struct ProtectiveMeasurePresenter {
    victim_read_repository: Arc<dyn VictimReadRepository>,
    offender_read_repository: Arc<dyn OffenderReadRepository>,
    city_repository: Arc<dyn CityRepository>,
}

impl ProtectiveMeasurePresenter {
    pub fn new(
        victim_read_repository: Arc<dyn VictimReadRepository>,
        offender_read_repository: Arc<dyn OffenderReadRepository>,
        city_repository: Arc<dyn CityRepository>,
    ) -> Self {
        Self {
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
        if include_related_entities {
            Ok(ProtectiveMeasureResponse::WithEntities(
                self.build_with_entities(measure).await?,
            ))
        } else {
            Ok(ProtectiveMeasureResponse::Simple(ProtectiveMeasureSimple {
                measure,
            }))
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
            victim: victim.into(),
            offender: offender.into(),
            court_district: court_district.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use chrono::Utc;
    use uuid::Uuid;

    use super::*;
    use crate::core::commands::cities::{CreateCity, UpdateCity};
    use crate::core::commands::offenders::{CreateOffender, UpdateOffender};
    use crate::core::commands::victims::{CreateVictim, UpdateVictim};
    use crate::core::contracts::repository::offenders::{
        OffenderReadRepository, OffenderWriteRepository,
    };
    use crate::core::contracts::repository::victims::{
        VictimReadRepository, VictimWriteRepository,
    };
    use crate::core::entities::cities::City;
    use crate::core::entities::common::EducationLevel;
    use crate::core::entities::common::{AddressData, PhoneData};
    use crate::core::entities::offenders::{OffenderAddress, OffenderPhone, OffenderWriteResult};
    use crate::core::entities::protective_measures::{
        ProtectiveMeasureStatus, RelationshipToVictim, ViolenceType,
    };
    use crate::core::entities::victims::{VictimAddress, VictimPhone, VictimWriteResult};
    use crate::core::read_models::offenders::OffenderWithDetails;
    use crate::core::read_models::victims::VictimWithDetails;

    struct FakeVictimRepo {
        victim: Option<VictimWithDetails>,
    }

    struct FakeOffenderRepo {
        offender: Option<OffenderWithDetails>,
    }

    struct FakeCityRepo {
        city: Option<City>,
    }

    #[async_trait]
    impl VictimReadRepository for FakeVictimRepo {
        async fn get_victim_by_id(&self, _id: Uuid) -> Result<VictimWithDetails, RepositoryError> {
            self.victim.clone().ok_or(RepositoryError::NotFound)
        }

        async fn get_all_victims(&self) -> Result<Vec<VictimWithDetails>, RepositoryError> {
            unimplemented!()
        }

        async fn get_victims_by_city(
            &self,
            _city_id: Uuid,
        ) -> Result<Vec<VictimWithDetails>, RepositoryError> {
            unimplemented!()
        }

        async fn get_victims_paginated<'a>(
            &'a self,
            _allowed_cities: Option<&'a [Uuid]>,
            _limit: i64,
            _offset: i64,
        ) -> Result<Vec<VictimWithDetails>, RepositoryError>
        where
            Self: 'a,
        {
            unimplemented!()
        }

        async fn count_victims<'a>(
            &'a self,
            _allowed_cities: Option<&'a [Uuid]>,
        ) -> Result<i64, RepositoryError>
        where
            Self: 'a,
        {
            unimplemented!()
        }

        async fn get_victims_by_name(
            &self,
            _name: &str,
        ) -> Result<Vec<VictimWithDetails>, RepositoryError> {
            unimplemented!()
        }

        async fn get_victims_by_cpf(
            &self,
            _cpf: &str,
        ) -> Result<Vec<VictimWithDetails>, RepositoryError> {
            unimplemented!()
        }
    }

    #[async_trait]
    impl VictimWriteRepository for FakeVictimRepo {
        async fn create_victim(
            &self,
            _victim: CreateVictim,
        ) -> Result<VictimWriteResult, RepositoryError> {
            unimplemented!()
        }

        async fn update_victim_by_id(
            &self,
            _data: UpdateVictim,
            _id: Uuid,
        ) -> Result<VictimWriteResult, RepositoryError> {
            unimplemented!()
        }

        async fn delete_victim_by_id(
            &self,
            _id: Uuid,
        ) -> Result<VictimWriteResult, RepositoryError> {
            unimplemented!()
        }

        async fn create_phone(
            &self,
            _victim_id: Uuid,
            _phone_data: PhoneData,
        ) -> Result<VictimPhone, RepositoryError> {
            unimplemented!()
        }

        async fn get_phone_by_id(&self, _phone_id: Uuid) -> Result<VictimPhone, RepositoryError> {
            unimplemented!()
        }

        async fn update_phone_by_id(
            &self,
            _phone_id: Uuid,
            _phone_data: PhoneData,
        ) -> Result<VictimPhone, RepositoryError> {
            unimplemented!()
        }

        async fn delete_phone_by_id(
            &self,
            _phone_id: Uuid,
        ) -> Result<VictimPhone, RepositoryError> {
            unimplemented!()
        }

        async fn create_address(
            &self,
            _victim_id: Uuid,
            _address_data: AddressData,
        ) -> Result<VictimAddress, RepositoryError> {
            unimplemented!()
        }

        async fn get_address_by_id(
            &self,
            _address_id: Uuid,
        ) -> Result<VictimAddress, RepositoryError> {
            unimplemented!()
        }

        async fn update_address_by_id(
            &self,
            _address_id: Uuid,
            _address_data: AddressData,
        ) -> Result<VictimAddress, RepositoryError> {
            unimplemented!()
        }

        async fn delete_address_by_id(
            &self,
            _address_id: Uuid,
        ) -> Result<VictimAddress, RepositoryError> {
            unimplemented!()
        }
    }

    #[async_trait]
    impl OffenderReadRepository for FakeOffenderRepo {
        async fn get_offender_by_id(
            &self,
            _id: Uuid,
        ) -> Result<OffenderWithDetails, RepositoryError> {
            self.offender.clone().ok_or(RepositoryError::NotFound)
        }

        async fn get_all_offenders(&self) -> Result<Vec<OffenderWithDetails>, RepositoryError> {
            unimplemented!()
        }

        async fn get_offenders_paginated(
            &self,
            _allowed_cities: Option<&[Uuid]>,
            _limit: i64,
            _offset: i64,
        ) -> Result<Vec<OffenderWithDetails>, RepositoryError> {
            unimplemented!()
        }

        async fn count_offenders(
            &self,
            _allowed_cities: Option<&[Uuid]>,
        ) -> Result<i64, RepositoryError> {
            unimplemented!()
        }

        async fn get_offenders_by_city(
            &self,
            _city_id: Uuid,
        ) -> Result<Vec<OffenderWithDetails>, RepositoryError> {
            unimplemented!()
        }

        async fn get_offenders_by_name(
            &self,
            _name: &str,
        ) -> Result<Vec<OffenderWithDetails>, RepositoryError> {
            unimplemented!()
        }

        async fn get_offenders_by_cpf(
            &self,
            _cpf: &str,
        ) -> Result<Vec<OffenderWithDetails>, RepositoryError> {
            unimplemented!()
        }

        async fn get_offenders_by_victim_id(
            &self,
            _victim_id: Uuid,
        ) -> Result<Vec<OffenderWithDetails>, RepositoryError> {
            unimplemented!()
        }
    }

    #[async_trait]
    impl OffenderWriteRepository for FakeOffenderRepo {
        async fn create_offender(
            &self,
            _offender: CreateOffender,
        ) -> Result<OffenderWriteResult, RepositoryError> {
            unimplemented!()
        }

        async fn update_offender_by_id(
            &self,
            _data: UpdateOffender,
            _id: Uuid,
        ) -> Result<OffenderWriteResult, RepositoryError> {
            unimplemented!()
        }

        async fn delete_offender_by_id(
            &self,
            _id: Uuid,
        ) -> Result<OffenderWriteResult, RepositoryError> {
            unimplemented!()
        }

        async fn create_phone(
            &self,
            _offender_id: Uuid,
            _phone_data: PhoneData,
        ) -> Result<OffenderPhone, RepositoryError> {
            unimplemented!()
        }

        async fn get_phone_by_id(&self, _phone_id: Uuid) -> Result<OffenderPhone, RepositoryError> {
            unimplemented!()
        }

        async fn update_phone_by_id(
            &self,
            _phone_id: Uuid,
            _phone_data: PhoneData,
        ) -> Result<OffenderPhone, RepositoryError> {
            unimplemented!()
        }

        async fn delete_phone_by_id(
            &self,
            _phone_id: Uuid,
        ) -> Result<OffenderPhone, RepositoryError> {
            unimplemented!()
        }

        async fn create_address(
            &self,
            _offender_id: Uuid,
            _address_data: AddressData,
        ) -> Result<OffenderAddress, RepositoryError> {
            unimplemented!()
        }

        async fn get_address_by_id(
            &self,
            _address_id: Uuid,
        ) -> Result<OffenderAddress, RepositoryError> {
            unimplemented!()
        }

        async fn update_address_by_id(
            &self,
            _address_id: Uuid,
            _address_data: AddressData,
        ) -> Result<OffenderAddress, RepositoryError> {
            unimplemented!()
        }

        async fn delete_address_by_id(
            &self,
            _address_id: Uuid,
        ) -> Result<OffenderAddress, RepositoryError> {
            unimplemented!()
        }
    }

    #[async_trait]
    impl CityRepository for FakeCityRepo {
        async fn create_city(&self, _city: CreateCity) -> Result<City, RepositoryError> {
            unimplemented!()
        }

        async fn get_city_by_id(&self, _id: Uuid) -> Result<City, RepositoryError> {
            self.city.clone().ok_or(RepositoryError::NotFound)
        }

        async fn get_all_cities(&self) -> Result<Vec<City>, RepositoryError> {
            unimplemented!()
        }

        async fn get_cities_paginated(
            &self,
            _allowed_cities: Option<&[Uuid]>,
            _limit: i64,
            _offset: i64,
        ) -> Result<Vec<City>, RepositoryError> {
            unimplemented!()
        }

        async fn count_cities(
            &self,
            _allowed_cities: Option<&[Uuid]>,
        ) -> Result<i64, RepositoryError> {
            unimplemented!()
        }

        async fn update_city_by_id(
            &self,
            _data: UpdateCity,
            _id: Uuid,
        ) -> Result<City, RepositoryError> {
            unimplemented!()
        }

        async fn delete_city_by_id(&self, _id: Uuid) -> Result<City, RepositoryError> {
            unimplemented!()
        }

        async fn get_city_by_name_and_battalion(
            &self,
            _name: &str,
            _battalion: &str,
        ) -> Result<Option<City>, RepositoryError> {
            unimplemented!()
        }
    }

    fn presenter(
        victim: Option<VictimWithDetails>,
        offender: Option<OffenderWithDetails>,
        city: Option<City>,
    ) -> ProtectiveMeasurePresenter {
        ProtectiveMeasurePresenter::new(
            Arc::new(FakeVictimRepo { victim }),
            Arc::new(FakeOffenderRepo { offender }),
            Arc::new(FakeCityRepo { city }),
        )
    }

    fn measure(victim_id: Uuid, offender_id: Uuid, city_id: Uuid) -> ProtectiveMeasure {
        let now = Utc::now();
        ProtectiveMeasure {
            id: Uuid::new_v4(),
            process_number: "12345-67.2025.8.26.0000".to_string(),
            sei_process_number: None,
            occurrence_report_number: None,
            issued_at: now.date_naive(),
            judicial_authority: "Juiz A".to_string(),
            court_district_id: city_id,
            distance_meters: None,
            status: ProtectiveMeasureStatus::Valid,
            violence_types: vec![ViolenceType::Physical],
            relationship_to_victim: RelationshipToVictim::Spouse,
            assaults_children: false,
            was_drunk_during_assault: false,
            victim_id,
            offender_id,
            created_at: now,
            updated_at: now,
            is_deleted: false,
        }
    }

    fn victim(id: Uuid, city_id: Uuid) -> VictimWithDetails {
        let now = Utc::now();
        VictimWithDetails {
            id,
            full_name: "MARIA TESTE".to_string(),
            cpf: None,
            birth_date: None,
            city_id,
            created_at: now,
            updated_at: now,
            is_deleted: false,
            education_level: None,
            occupation: None,
            has_children: false,
            children_count: None,
            is_pregnant: None,
            has_special_needs: false,
            special_needs_type: None,
            uses_alcohol: false,
            uses_drugs: false,
            has_psychiatric_issues: false,
            psychiatric_issues_type: None,
            phones: vec![],
            addresses: vec![],
        }
    }

    fn offender(id: Uuid, city_id: Uuid) -> OffenderWithDetails {
        let now = Utc::now();
        OffenderWithDetails {
            id,
            full_name: "JOAO TESTE".to_string(),
            cpf: None,
            birth_date: None,
            city_id,
            imprisoned: false,
            occupation: None,
            is_public_security_agent: false,
            security_force: None,
            uses_alcohol: false,
            uses_drugs: false,
            has_psychiatric_issues: false,
            psychiatric_issues_type: None,
            education_level: EducationLevel::HighSchool,
            observation: None,
            created_at: now,
            updated_at: now,
            is_deleted: false,
            phones: vec![],
            addresses: vec![],
        }
    }

    fn city(id: Uuid) -> City {
        let now = Utc::now();
        City {
            id,
            name: "Cidade Teste".to_string(),
            state: "RO".to_string(),
            battalion: "1 BPM".to_string(),
            created_at: now,
            updated_at: now,
            is_deleted: false,
        }
    }

    #[actix_rt::test]
    async fn build_response_without_include_related_omits_related_entities() {
        let victim_id = Uuid::new_v4();
        let offender_id = Uuid::new_v4();
        let city_id = Uuid::new_v4();
        let measure = measure(victim_id, offender_id, city_id);
        let presenter = presenter(None, None, None);

        let response = presenter
            .build_response(measure.clone(), false)
            .await
            .unwrap();

        match response {
            ProtectiveMeasureResponse::Simple(simple) => {
                assert_eq!(simple.measure.id, measure.id);
                assert_eq!(simple.measure.victim_id, victim_id);
                assert_eq!(simple.measure.offender_id, offender_id);
            }
            ProtectiveMeasureResponse::WithEntities(_) => panic!("unexpected related response"),
        }
    }

    #[actix_rt::test]
    async fn build_response_with_include_related_populates_victim_and_offender() {
        let victim_id = Uuid::new_v4();
        let offender_id = Uuid::new_v4();
        let city_id = Uuid::new_v4();
        let presenter = presenter(
            Some(victim(victim_id, city_id)),
            Some(offender(offender_id, city_id)),
            Some(city(city_id)),
        );

        let response = presenter
            .build_response(measure(victim_id, offender_id, city_id), true)
            .await
            .unwrap();

        match response {
            ProtectiveMeasureResponse::WithEntities(with_entities) => {
                assert_eq!(with_entities.victim.full_name, "MARIA TESTE");
                assert_eq!(with_entities.offender.full_name, "JOAO TESTE");
                assert_eq!(with_entities.court_district.name, "Cidade Teste");
            }
            ProtectiveMeasureResponse::Simple(_) => panic!("expected related response"),
        }
    }

    #[actix_rt::test]
    async fn build_response_with_include_related_and_missing_court_district_does_not_panic() {
        let victim_id = Uuid::new_v4();
        let offender_id = Uuid::new_v4();
        let city_id = Uuid::new_v4();
        let presenter = presenter(
            Some(victim(victim_id, city_id)),
            Some(offender(offender_id, city_id)),
            None,
        );

        let error = presenter
            .build_response(measure(victim_id, offender_id, city_id), true)
            .await
            .unwrap_err();

        assert!(matches!(error, AppError::NotFound(_)));
    }

    #[actix_rt::test]
    async fn build_response_with_null_optional_fields_does_not_panic() {
        let victim_id = Uuid::new_v4();
        let offender_id = Uuid::new_v4();
        let city_id = Uuid::new_v4();
        let mut measure = measure(victim_id, offender_id, city_id);
        measure.sei_process_number = None;
        measure.occurrence_report_number = None;
        measure.distance_meters = None;
        let presenter = presenter(None, None, None);

        let response = presenter.build_response(measure, false).await.unwrap();

        assert!(matches!(response, ProtectiveMeasureResponse::Simple(_)));
    }
}
