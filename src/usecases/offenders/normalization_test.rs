use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::commands::offenders::SaveOffender;
use crate::core::entities::common::{AddressData, AddressType, EducationLevel};
use crate::usecases::offenders::normalization::normalize_offender_input;

fn base_command(city_id: Option<Uuid>, full_name: &str, cpf: Option<&str>) -> SaveOffender {
    SaveOffender {
        full_name: full_name.to_string(),
        cpf: cpf.map(|s| s.to_string()),
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
        phones: None,
        addresses: None,
    }
}

fn residential_address(city_id: Uuid) -> AddressData {
    AddressData {
        street: Some("Rua Teste".to_string()),
        number: Some("100".to_string()),
        district: None,
        city_id,
        zip_code: None,
        complement: None,
        address_type: AddressType::Residential,
    }
}

#[test]
fn normalize_uppercases_full_name() {
    let city_id = Uuid::new_v4();
    let mut cmd = base_command(Some(city_id), "maria silva", None);

    normalize_offender_input(&mut cmd, "ctx").unwrap();

    assert_eq!(cmd.full_name, "MARIA SILVA");
}

#[test]
fn normalize_preserves_accents_in_full_name() {
    let city_id = Uuid::new_v4();
    let mut cmd = base_command(Some(city_id), "joão da silva", None);

    normalize_offender_input(&mut cmd, "ctx").unwrap();

    assert_eq!(cmd.full_name, "JOÃO DA SILVA");
}

#[test]
fn normalize_trims_full_name() {
    let city_id = Uuid::new_v4();
    let mut cmd = base_command(Some(city_id), "  João  ", None);

    normalize_offender_input(&mut cmd, "ctx").unwrap();

    assert_eq!(cmd.full_name, "JOÃO");
}

#[test]
fn normalize_preserves_cpf_mask_when_valid() {
    let city_id = Uuid::new_v4();
    let mut cmd = base_command(Some(city_id), "Valid User", Some("529.982.247-25"));

    normalize_offender_input(&mut cmd, "ctx").unwrap();

    assert_eq!(cmd.cpf.as_deref(), Some("529.982.247-25"));
}

#[test]
fn normalize_rejects_cpf_without_mask() {
    let city_id = Uuid::new_v4();
    let mut cmd = base_command(Some(city_id), "User", Some("52998224725"));

    let err = normalize_offender_input(&mut cmd, "ctx").unwrap_err();

    assert!(
        matches!(err, AppError::BadRequest(msg) if msg.contains("14 characters")),
        "expected 14-char length error, got other BadRequest"
    );
}

#[test]
fn normalize_rejects_cpf_with_invalid_check_digits() {
    let city_id = Uuid::new_v4();
    let mut cmd = base_command(Some(city_id), "User", Some("111.111.111-11"));

    let err = normalize_offender_input(&mut cmd, "ctx").unwrap_err();

    assert!(
        matches!(err, AppError::BadRequest(msg) if msg.contains("invalid check digits")),
        "expected invalid check digits error"
    );
}

#[test]
fn normalize_applies_city_derivation_from_residential_address_when_city_id_absent() {
    let derived_city_id = Uuid::new_v4();
    let mut cmd = base_command(None, "User", None);
    cmd.addresses = Some(vec![residential_address(derived_city_id)]);

    let resolved = normalize_offender_input(&mut cmd, "ctx").unwrap();

    assert_eq!(resolved, derived_city_id);
    assert_eq!(cmd.city_id, Some(derived_city_id));
}

#[test]
fn normalize_fails_when_no_city_id_and_no_addresses() {
    let mut cmd = base_command(None, "User", None);

    let err = normalize_offender_input(&mut cmd, "ctx").unwrap_err();

    assert!(
        matches!(err, AppError::BadRequest(msg) if msg.contains("city_id")),
        "expected missing city_id BadRequest"
    );
}

#[test]
fn normalize_derives_psychiatric_flag_from_list() {
    let city_id = Uuid::new_v4();
    let mut cmd = base_command(Some(city_id), "User", None);
    cmd.has_psychiatric_issues = false;
    cmd.psychiatric_issues_type = Some(vec!["Depression".to_string()]);

    normalize_offender_input(&mut cmd, "ctx").unwrap();

    assert!(cmd.has_psychiatric_issues);
    assert_eq!(
        cmd.psychiatric_issues_type.as_deref(),
        Some(&["Depression".to_string()][..])
    );
}

#[test]
fn normalize_clears_psychiatric_flag_when_list_is_empty() {
    let city_id = Uuid::new_v4();
    let mut cmd = base_command(Some(city_id), "User", None);
    cmd.has_psychiatric_issues = true;
    cmd.psychiatric_issues_type = Some(vec![]);

    normalize_offender_input(&mut cmd, "ctx").unwrap();

    assert!(!cmd.has_psychiatric_issues);
    assert!(cmd.psychiatric_issues_type.is_none());
}
