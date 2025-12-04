use async_trait::async_trait;
use log::info;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::querys::offenders::{
    OffenderAddressesQueries, OffenderPhonesQueries, OffenderWorkAddressesQueries,
    OffendersQueries,
};
use crate::core::contracts::repository::offenders::OffenderRepository;
use crate::core::entities::offenders::{
    AddressData, CreateOffender, Offender, OffenderAddress, OffenderPhone, OffenderWithDetails,
    OffenderWorkAddress, PhoneData, UpdateOffender, WorkAddressData,
};

#[derive(Clone)]
pub struct PgOffenderRepository {
    pool: PgPool,
}

impl PgOffenderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn create_phone_internal(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        offender_id: Uuid,
        phone_data: &PhoneData,
    ) -> Result<OffenderPhone, sqlx::Error> {
        let phone_id = Uuid::new_v4();

        let created: OffenderPhone = sqlx::query_as(OffenderPhonesQueries::CREATE_OFFENDER_PHONE)
            .bind(phone_id)
            .bind(offender_id)
            .bind(&phone_data.phone)
            .bind(&phone_data.phone_type)
            .fetch_one(&mut **tx)
            .await?;

        Ok(created)
    }

    async fn delete_phones_by_offender_id(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        offender_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        let _: Vec<OffenderPhone> =
            sqlx::query_as(OffenderPhonesQueries::DELETE_OFFENDER_PHONES_BY_OFFENDER_ID)
                .bind(offender_id)
                .fetch_all(&mut **tx)
                .await?;
        Ok(())
    }

    async fn create_address_internal(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        offender_id: Uuid,
        address: &AddressData,
    ) -> Result<OffenderAddress, sqlx::Error> {
        let address_id = Uuid::new_v4();

        let created: OffenderAddress =
            sqlx::query_as(OffenderAddressesQueries::CREATE_OFFENDER_ADDRESS)
                .bind(address_id)
                .bind(offender_id)
                .bind(&address.street)
                .bind(&address.number)
                .bind(&address.district)
                .bind(&address.city_id)
                .bind(&address.zip_code)
                .bind(&address.complement)
                .fetch_one(&mut **tx)
                .await?;

        Ok(created)
    }

    async fn delete_addresses_by_offender_id(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        offender_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        let _: Vec<OffenderAddress> =
            sqlx::query_as(OffenderAddressesQueries::DELETE_OFFENDER_ADDRESSES_BY_OFFENDER_ID)
                .bind(offender_id)
                .fetch_all(&mut **tx)
                .await?;
        Ok(())
    }

    async fn create_work_address_internal(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        offender_id: Uuid,
        work_address: &WorkAddressData,
    ) -> Result<OffenderWorkAddress, sqlx::Error> {
        let work_address_id = Uuid::new_v4();

        let created: OffenderWorkAddress =
            sqlx::query_as(OffenderWorkAddressesQueries::CREATE_OFFENDER_WORK_ADDRESS)
                .bind(work_address_id)
                .bind(offender_id)
                .bind(&work_address.street)
                .bind(&work_address.number)
                .bind(&work_address.district)
                .bind(&work_address.city_id)
                .bind(&work_address.zip_code)
                .bind(&work_address.complement)
                .fetch_one(&mut **tx)
                .await?;

        Ok(created)
    }

    async fn delete_work_addresses_by_offender_id(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        offender_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        let _: Vec<OffenderWorkAddress> = sqlx::query_as(
            OffenderWorkAddressesQueries::DELETE_OFFENDER_WORK_ADDRESSES_BY_OFFENDER_ID,
        )
        .bind(offender_id)
        .fetch_all(&mut **tx)
        .await?;
        Ok(())
    }

    async fn get_phones_by_offender_id(
        &self,
        offender_id: Uuid,
    ) -> Result<Vec<OffenderPhone>, sqlx::Error> {
        let phones: Vec<OffenderPhone> =
            sqlx::query_as(OffenderPhonesQueries::GET_OFFENDER_PHONES_BY_OFFENDER_ID)
                .bind(offender_id)
                .fetch_all(&self.pool)
                .await?;
        Ok(phones)
    }

    async fn get_addresses_by_offender_id(
        &self,
        offender_id: Uuid,
    ) -> Result<Vec<OffenderAddress>, sqlx::Error> {
        let addresses: Vec<OffenderAddress> =
            sqlx::query_as(OffenderAddressesQueries::GET_OFFENDER_ADDRESSES_BY_OFFENDER_ID)
                .bind(offender_id)
                .fetch_all(&self.pool)
                .await?;
        Ok(addresses)
    }

    async fn get_work_addresses_by_offender_id(
        &self,
        offender_id: Uuid,
    ) -> Result<Vec<OffenderWorkAddress>, sqlx::Error> {
        let work_addresses: Vec<OffenderWorkAddress> = sqlx::query_as(
            OffenderWorkAddressesQueries::GET_OFFENDER_WORK_ADDRESSES_BY_OFFENDER_ID,
        )
        .bind(offender_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(work_addresses)
    }
}

#[async_trait]
impl OffenderRepository for PgOffenderRepository {
    async fn create_offender(
        &self,
        offender: CreateOffender,
    ) -> Result<OffenderWithDetails, sqlx::Error> {
        let offender_id = Uuid::new_v4();

        info!(
            "[Repository] Starting transaction to create offender: {} with ID: {}",
            offender.full_name, offender_id
        );

        let mut tx = self.pool.begin().await?;

        let offender_created: Offender = sqlx::query_as(OffendersQueries::CREATE_OFFENDER)
            .bind(offender_id)
            .bind(&offender.full_name)
            .bind(&offender.cpf)
            .bind(&offender.birth_date)
            .bind(&offender.city_id)
            .bind(&offender.victim_id)
            .bind(&offender.imprisoned)
            .bind(&offender.occupation)
            .bind(&offender.workplace)
            .bind(&offender.is_public_security_agent)
            .bind(&offender.security_force)
            .bind(&offender.relationship_to_victim)
            .bind(&offender.uses_alcohol)
            .bind(&offender.uses_drugs)
            .bind(&offender.has_psychiatric_issues)
            .bind(&offender.psychiatric_issues_type)
            .bind(&offender.was_drunk_during_assault)
            .bind(&offender.observation)
            .fetch_one(&mut *tx)
            .await?;

        info!("[Repository] Offender inserted, now creating phones if provided");

        let mut created_phones = Vec::new();
        if let Some(phones) = &offender.phones {
            for phone_data in phones {
                let phone = Self::create_phone_internal(&mut tx, offender_id, phone_data).await?;
                created_phones.push(phone);
            }
            info!("[Repository] Created {} phone(s)", created_phones.len());
        }

        info!("[Repository] Now creating addresses if provided");

        let mut created_addresses = Vec::new();
        if let Some(addresses) = &offender.addresses {
            for addr_data in addresses {
                let created_addr =
                    Self::create_address_internal(&mut tx, offender_id, addr_data).await?;
                created_addresses.push(created_addr);
            }
            info!(
                "[Repository] Created {} address(es)",
                created_addresses.len()
            );
        }

        info!("[Repository] Now creating work addresses if provided");

        let mut created_work_addresses = Vec::new();
        if let Some(work_addresses) = &offender.work_addresses {
            for work_addr_data in work_addresses {
                let created_work_addr =
                    Self::create_work_address_internal(&mut tx, offender_id, work_addr_data)
                        .await?;
                created_work_addresses.push(created_work_addr);
            }
            info!(
                "[Repository] Created {} work address(es)",
                created_work_addresses.len()
            );
        }

        tx.commit().await?;

        info!(
            "[Repository] Transaction committed. Offender {} created successfully",
            offender_id
        );

        Ok(offender_created.with_details(
            created_phones,
            created_addresses,
            created_work_addresses,
        ))
    }

    async fn get_offender_by_id(&self, id: Uuid) -> Result<OffenderWithDetails, sqlx::Error> {
        info!("[Repository] Fetching offender with id: {}", id);

        let offender: Offender = sqlx::query_as(OffendersQueries::GET_OFFENDER_BY_ID)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        let phones = self.get_phones_by_offender_id(id).await?;
        let addresses = self.get_addresses_by_offender_id(id).await?;
        let work_addresses = self.get_work_addresses_by_offender_id(id).await?;

        info!(
            "[Repository] Offender {} found with {} phone(s), {} address(es), and {} work address(es)",
            id,
            phones.len(),
            addresses.len(),
            work_addresses.len()
        );

        Ok(offender.with_details(phones, addresses, work_addresses))
    }

    async fn get_all_offenders(&self) -> Result<Vec<OffenderWithDetails>, sqlx::Error> {
        info!("[Repository] Fetching all offenders");

        let offenders: Vec<Offender> = sqlx::query_as(OffendersQueries::GET_ALL_OFFENDERS)
            .fetch_all(&self.pool)
            .await?;

        let mut result = Vec::with_capacity(offenders.len());

        for offender in offenders {
            let phones = self.get_phones_by_offender_id(offender.id).await?;
            let addresses = self.get_addresses_by_offender_id(offender.id).await?;
            let work_addresses = self.get_work_addresses_by_offender_id(offender.id).await?;
            result.push(offender.with_details(phones, addresses, work_addresses));
        }

        info!("[Repository] Found {} offenders", result.len());

        Ok(result)
    }

    async fn get_offenders_by_city(
        &self,
        city_id: Uuid,
    ) -> Result<Vec<OffenderWithDetails>, sqlx::Error> {
        info!("[Repository] Fetching offenders for city: {}", city_id);

        let offenders: Vec<Offender> = sqlx::query_as(OffendersQueries::GET_OFFENDERS_BY_CITY)
            .bind(city_id)
            .fetch_all(&self.pool)
            .await?;

        let mut result = Vec::with_capacity(offenders.len());

        for offender in offenders {
            let phones = self.get_phones_by_offender_id(offender.id).await?;
            let addresses = self.get_addresses_by_offender_id(offender.id).await?;
            let work_addresses = self.get_work_addresses_by_offender_id(offender.id).await?;
            result.push(offender.with_details(phones, addresses, work_addresses));
        }

        info!(
            "[Repository] Found {} offenders for city: {}",
            result.len(),
            city_id
        );

        Ok(result)
    }

    async fn get_offenders_by_victim_id(
        &self,
        victim_id: Uuid,
    ) -> Result<Vec<OffenderWithDetails>, sqlx::Error> {
        info!("[Repository] Fetching offenders for victim: {}", victim_id);

        let offenders: Vec<Offender> =
            sqlx::query_as(OffendersQueries::GET_OFFENDERS_BY_VICTIM_ID)
                .bind(victim_id)
                .fetch_all(&self.pool)
                .await?;

        let mut result = Vec::with_capacity(offenders.len());

        for offender in offenders {
            let phones = self.get_phones_by_offender_id(offender.id).await?;
            let addresses = self.get_addresses_by_offender_id(offender.id).await?;
            let work_addresses = self.get_work_addresses_by_offender_id(offender.id).await?;
            result.push(offender.with_details(phones, addresses, work_addresses));
        }

        info!(
            "[Repository] Found {} offenders for victim: {}",
            result.len(),
            victim_id
        );

        Ok(result)
    }

    async fn update_offender_by_id(
        &self,
        data: UpdateOffender,
        id: Uuid,
    ) -> Result<OffenderWithDetails, sqlx::Error> {
        info!("[Repository] Starting transaction to update offender: {}", id);

        let mut tx = self.pool.begin().await?;

        let offender_updated: Offender = sqlx::query_as(OffendersQueries::UPDATE_OFFENDER_BY_ID)
            .bind(id)
            .bind(&data.full_name)
            .bind(&data.cpf)
            .bind(&data.birth_date)
            .bind(&data.city_id)
            .bind(&data.victim_id)
            .bind(&data.imprisoned)
            .bind(&data.occupation)
            .bind(&data.workplace)
            .bind(&data.is_public_security_agent)
            .bind(&data.security_force)
            .bind(&data.relationship_to_victim)
            .bind(&data.uses_alcohol)
            .bind(&data.uses_drugs)
            .bind(&data.has_psychiatric_issues)
            .bind(&data.psychiatric_issues_type)
            .bind(&data.was_drunk_during_assault)
            .bind(&data.observation)
            .fetch_one(&mut *tx)
            .await?;

        Self::delete_phones_by_offender_id(&mut tx, id).await?;
        let mut updated_phones = Vec::new();
        if let Some(phones) = &data.phones {
            for phone_data in phones {
                let phone = Self::create_phone_internal(&mut tx, id, phone_data).await?;
                updated_phones.push(phone);
            }
            info!(
                "[Repository] Updated {} phone(s) for offender: {}",
                phones.len(),
                id
            );
        }

        Self::delete_addresses_by_offender_id(&mut tx, id).await?;
        let mut updated_addresses = Vec::new();
        if let Some(addresses) = &data.addresses {
            for addr_data in addresses {
                let created_addr = Self::create_address_internal(&mut tx, id, addr_data).await?;
                updated_addresses.push(created_addr);
            }
            info!(
                "[Repository] Updated {} address(es) for offender: {}",
                addresses.len(),
                id
            );
        }

        Self::delete_work_addresses_by_offender_id(&mut tx, id).await?;
        let mut updated_work_addresses = Vec::new();
        if let Some(work_addresses) = &data.work_addresses {
            for work_addr_data in work_addresses {
                let created_work_addr =
                    Self::create_work_address_internal(&mut tx, id, work_addr_data).await?;
                updated_work_addresses.push(created_work_addr);
            }
            info!(
                "[Repository] Updated {} work address(es) for offender: {}",
                work_addresses.len(),
                id
            );
        }

        tx.commit().await?;

        info!(
            "[Repository] Transaction committed. Offender {} updated",
            id
        );

        Ok(offender_updated.with_details(
            updated_phones,
            updated_addresses,
            updated_work_addresses,
        ))
    }

    async fn delete_offender_by_id(&self, id: Uuid) -> Result<OffenderWithDetails, sqlx::Error> {
        info!(
            "[Repository] Starting transaction to soft delete offender: {}",
            id
        );

        let phones = self.get_phones_by_offender_id(id).await?;
        let addresses = self.get_addresses_by_offender_id(id).await?;
        let work_addresses = self.get_work_addresses_by_offender_id(id).await?;

        let mut tx = self.pool.begin().await?;

        Self::delete_phones_by_offender_id(&mut tx, id).await?;
        Self::delete_addresses_by_offender_id(&mut tx, id).await?;
        Self::delete_work_addresses_by_offender_id(&mut tx, id).await?;

        let deleted_offender: Offender = sqlx::query_as(OffendersQueries::DELETE_OFFENDER_BY_ID)
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;

        tx.commit().await?;

        info!(
            "[Repository] Transaction committed. Offender {} soft deleted",
            id
        );

        Ok(deleted_offender.with_details(phones, addresses, work_addresses))
    }

    async fn create_phone(
        &self,
        offender_id: Uuid,
        phone_data: PhoneData,
    ) -> Result<OffenderPhone, sqlx::Error> {
        info!("[Repository] Creating phone for offender: {}", offender_id);
        let phone_id = Uuid::new_v4();

        let phone: OffenderPhone = sqlx::query_as(OffenderPhonesQueries::CREATE_OFFENDER_PHONE)
            .bind(phone_id)
            .bind(offender_id)
            .bind(&phone_data.phone)
            .bind(&phone_data.phone_type)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Phone {} created successfully", phone_id);
        Ok(phone)
    }

    async fn get_phone_by_id(&self, phone_id: Uuid) -> Result<OffenderPhone, sqlx::Error> {
        info!("[Repository] Fetching phone with id: {}", phone_id);

        let phone: OffenderPhone = sqlx::query_as(OffenderPhonesQueries::GET_OFFENDER_PHONE_BY_ID)
            .bind(phone_id)
            .fetch_one(&self.pool)
            .await?;

        info!("[Repository] Phone {} found", phone_id);
        Ok(phone)
    }

    async fn update_phone_by_id(
        &self,
        phone_id: Uuid,
        phone_data: PhoneData,
    ) -> Result<OffenderPhone, sqlx::Error> {
        info!("[Repository] Updating phone: {}", phone_id);

        let phone: OffenderPhone =
            sqlx::query_as(OffenderPhonesQueries::UPDATE_OFFENDER_PHONE_BY_ID)
                .bind(phone_id)
                .bind(&phone_data.phone)
                .bind(&phone_data.phone_type)
                .fetch_one(&self.pool)
                .await?;

        info!("[Repository] Phone {} updated successfully", phone_id);
        Ok(phone)
    }

    async fn delete_phone_by_id(&self, phone_id: Uuid) -> Result<OffenderPhone, sqlx::Error> {
        info!("[Repository] Soft deleting phone: {}", phone_id);

        let phone: OffenderPhone =
            sqlx::query_as(OffenderPhonesQueries::DELETE_OFFENDER_PHONE_BY_ID)
                .bind(phone_id)
                .fetch_one(&self.pool)
                .await?;

        info!("[Repository] Phone {} soft deleted successfully", phone_id);
        Ok(phone)
    }

    async fn create_address(
        &self,
        offender_id: Uuid,
        address_data: AddressData,
    ) -> Result<OffenderAddress, sqlx::Error> {
        info!(
            "[Repository] Creating address for offender: {}",
            offender_id
        );
        let address_id = Uuid::new_v4();

        let address: OffenderAddress =
            sqlx::query_as(OffenderAddressesQueries::CREATE_OFFENDER_ADDRESS)
                .bind(address_id)
                .bind(offender_id)
                .bind(&address_data.street)
                .bind(&address_data.number)
                .bind(&address_data.district)
                .bind(&address_data.city_id)
                .bind(&address_data.zip_code)
                .bind(&address_data.complement)
                .fetch_one(&self.pool)
                .await?;

        info!("[Repository] Address {} created successfully", address_id);
        Ok(address)
    }

    async fn get_address_by_id(&self, address_id: Uuid) -> Result<OffenderAddress, sqlx::Error> {
        info!("[Repository] Fetching address with id: {}", address_id);

        let address: OffenderAddress =
            sqlx::query_as(OffenderAddressesQueries::GET_OFFENDER_ADDRESS_BY_ID)
                .bind(address_id)
                .fetch_one(&self.pool)
                .await?;

        info!("[Repository] Address {} found", address_id);
        Ok(address)
    }

    async fn update_address_by_id(
        &self,
        address_id: Uuid,
        address_data: AddressData,
    ) -> Result<OffenderAddress, sqlx::Error> {
        info!("[Repository] Updating address: {}", address_id);

        let address: OffenderAddress =
            sqlx::query_as(OffenderAddressesQueries::UPDATE_OFFENDER_ADDRESS_BY_ID)
                .bind(address_id)
                .bind(&address_data.street)
                .bind(&address_data.number)
                .bind(&address_data.district)
                .bind(&address_data.city_id)
                .bind(&address_data.zip_code)
                .bind(&address_data.complement)
                .fetch_one(&self.pool)
                .await?;

        info!("[Repository] Address {} updated successfully", address_id);
        Ok(address)
    }

    async fn delete_address_by_id(&self, address_id: Uuid) -> Result<OffenderAddress, sqlx::Error> {
        info!("[Repository] Soft deleting address: {}", address_id);

        let address: OffenderAddress =
            sqlx::query_as(OffenderAddressesQueries::DELETE_OFFENDER_ADDRESS_BY_ID)
                .bind(address_id)
                .fetch_one(&self.pool)
                .await?;

        info!("[Repository] Address {} soft deleted successfully", address_id);
        Ok(address)
    }

    async fn create_work_address(
        &self,
        offender_id: Uuid,
        work_address_data: WorkAddressData,
    ) -> Result<OffenderWorkAddress, sqlx::Error> {
        info!(
            "[Repository] Creating work address for offender: {}",
            offender_id
        );
        let work_address_id = Uuid::new_v4();

        let work_address: OffenderWorkAddress =
            sqlx::query_as(OffenderWorkAddressesQueries::CREATE_OFFENDER_WORK_ADDRESS)
                .bind(work_address_id)
                .bind(offender_id)
                .bind(&work_address_data.street)
                .bind(&work_address_data.number)
                .bind(&work_address_data.district)
                .bind(&work_address_data.city_id)
                .bind(&work_address_data.zip_code)
                .bind(&work_address_data.complement)
                .fetch_one(&self.pool)
                .await?;

        info!(
            "[Repository] Work address {} created successfully",
            work_address_id
        );
        Ok(work_address)
    }

    async fn get_work_address_by_id(
        &self,
        work_address_id: Uuid,
    ) -> Result<OffenderWorkAddress, sqlx::Error> {
        info!(
            "[Repository] Fetching work address with id: {}",
            work_address_id
        );

        let work_address: OffenderWorkAddress =
            sqlx::query_as(OffenderWorkAddressesQueries::GET_OFFENDER_WORK_ADDRESS_BY_ID)
                .bind(work_address_id)
                .fetch_one(&self.pool)
                .await?;

        info!("[Repository] Work address {} found", work_address_id);
        Ok(work_address)
    }

    async fn update_work_address_by_id(
        &self,
        work_address_id: Uuid,
        work_address_data: WorkAddressData,
    ) -> Result<OffenderWorkAddress, sqlx::Error> {
        info!("[Repository] Updating work address: {}", work_address_id);

        let work_address: OffenderWorkAddress =
            sqlx::query_as(OffenderWorkAddressesQueries::UPDATE_OFFENDER_WORK_ADDRESS_BY_ID)
                .bind(work_address_id)
                .bind(&work_address_data.street)
                .bind(&work_address_data.number)
                .bind(&work_address_data.district)
                .bind(&work_address_data.city_id)
                .bind(&work_address_data.zip_code)
                .bind(&work_address_data.complement)
                .fetch_one(&self.pool)
                .await?;

        info!(
            "[Repository] Work address {} updated successfully",
            work_address_id
        );
        Ok(work_address)
    }

    async fn delete_work_address_by_id(
        &self,
        work_address_id: Uuid,
    ) -> Result<OffenderWorkAddress, sqlx::Error> {
        info!(
            "[Repository] Soft deleting work address: {}",
            work_address_id
        );

        let work_address: OffenderWorkAddress =
            sqlx::query_as(OffenderWorkAddressesQueries::DELETE_OFFENDER_WORK_ADDRESS_BY_ID)
                .bind(work_address_id)
                .fetch_one(&self.pool)
                .await?;

        info!(
            "[Repository] Work address {} soft deleted successfully",
            work_address_id
        );
        Ok(work_address)
    }
}
