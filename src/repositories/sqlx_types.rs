use sqlx::encode::IsNull;
use sqlx::postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef, Postgres};
use sqlx::{Decode, Encode, Type};

use crate::core::entities::attendance_offenders::ViolenceAggravator;
use crate::core::entities::attendance_victims::{
    OffenderFirearmAccess, OffenderFreedomStatus, RiskLevel,
};
use crate::core::entities::common::{AddressType, EducationLevel, PhoneType};
use crate::core::entities::offenders::SecurityForce;
use crate::core::entities::protective_measures::{
    ProtectiveMeasureStatus, RelationshipToVictim, ViolenceType,
};
use crate::core::entities::work_session_members::TeamMemberFunction;
use crate::core::value_objects::profiles::Profile;
use crate::core::value_objects::ranks::Rank;

macro_rules! impl_postgres_enum_type {
    ($ty:path, $type_name:literal) => {
        impl Type<Postgres> for $ty {
            fn type_info() -> PgTypeInfo {
                PgTypeInfo::with_name($type_name)
            }
        }

        impl PgHasArrayType for $ty {
            fn array_type_info() -> PgTypeInfo {
                PgTypeInfo::array_of($type_name)
            }
        }

        impl<'q> Encode<'q, Postgres> for $ty {
            fn encode_by_ref(
                &self,
                buf: &mut PgArgumentBuffer,
            ) -> Result<IsNull, sqlx::error::BoxDynError> {
                <&str as Encode<Postgres>>::encode(self.as_str(), buf)
            }

            fn size_hint(&self) -> usize {
                <&str as Encode<Postgres>>::size_hint(&self.as_str())
            }
        }

        impl<'r> Decode<'r, Postgres> for $ty {
            fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
                let value = <&'r str as Decode<Postgres>>::decode(value)?;
                <$ty>::try_from(value).map_err(|error| error.into())
            }
        }
    };
}

macro_rules! impl_postgres_string_backed_enum {
    ($ty:path) => {
        impl Type<Postgres> for $ty {
            fn type_info() -> PgTypeInfo {
                <String as Type<Postgres>>::type_info()
            }

            fn compatible(ty: &PgTypeInfo) -> bool {
                <String as Type<Postgres>>::compatible(ty)
            }
        }

        impl<'q> Encode<'q, Postgres> for $ty {
            fn encode_by_ref(
                &self,
                buf: &mut PgArgumentBuffer,
            ) -> Result<IsNull, sqlx::error::BoxDynError> {
                <&str as Encode<Postgres>>::encode(self.as_str(), buf)
            }

            fn size_hint(&self) -> usize {
                <&str as Encode<Postgres>>::size_hint(&self.as_str())
            }
        }

        impl<'r> Decode<'r, Postgres> for $ty {
            fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
                let value = <&'r str as Decode<Postgres>>::decode(value)?;
                <$ty>::try_from(value).map_err(|error| error.into())
            }
        }
    };
}

impl_postgres_string_backed_enum!(Profile);
impl_postgres_string_backed_enum!(Rank);

impl_postgres_enum_type!(PhoneType, "phone_type_enum");
impl_postgres_enum_type!(AddressType, "address_type_enum");
impl_postgres_enum_type!(EducationLevel, "education_level_enum");
impl_postgres_enum_type!(SecurityForce, "security_force_enum");
impl_postgres_enum_type!(TeamMemberFunction, "team_member_function");
impl_postgres_enum_type!(ViolenceType, "violence_type_enum");
impl_postgres_enum_type!(RelationshipToVictim, "relationship_to_victim_enum");
impl_postgres_enum_type!(ProtectiveMeasureStatus, "protective_measure_status_enum");
impl_postgres_enum_type!(RiskLevel, "risk_level");
impl_postgres_enum_type!(OffenderFreedomStatus, "offender_freedom_status");
impl_postgres_enum_type!(OffenderFirearmAccess, "offender_firearm_access");
impl_postgres_enum_type!(ViolenceAggravator, "violence_aggravator_enum");
