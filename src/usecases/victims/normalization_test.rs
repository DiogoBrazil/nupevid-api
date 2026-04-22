use uuid::Uuid;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::commands::victims::SaveVictim;
use crate::core::entities::common::{AddressData, AddressType};
use crate::usecases::victims::normalization::normalize_victim_input;

fn base_command(city_id: Option<Uuid>, full_name: &str, cpf: Option<&str>) -> SaveVictim {
    SaveVictim {
        full_name: full_name.to_string(),
        cpf: cpf.map(|s| s.to_string()),
        birth_date: None,
        city_id,
        phones: None,
        addresses: None,
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

    normalize_victim_input(&mut cmd, "ctx").unwrap();

    assert_eq!(cmd.full_name, "MARIA SILVA");
}

#[test]
fn normalize_preserves_accents_in_full_name() {
    let city_id = Uuid::new_v4();
    let mut cmd = base_command(Some(city_id), "joão da silva", None);

    normalize_victim_input(&mut cmd, "ctx").unwrap();

    assert_eq!(cmd.full_name, "JOÃO DA SILVA");
}

#[test]
fn normalize_trims_full_name() {
    let city_id = Uuid::new_v4();
    let mut cmd = base_command(Some(city_id), "  João  ", None);

    normalize_victim_input(&mut cmd, "ctx").unwrap();

    assert_eq!(cmd.full_name, "JOÃO");
}

#[test]
fn normalize_preserves_cpf_mask_when_valid() {
    let city_id = Uuid::new_v4();
    let mut cmd = base_command(Some(city_id), "Valid User", Some("529.982.247-25"));

    normalize_victim_input(&mut cmd, "ctx").unwrap();

    assert_eq!(cmd.cpf.as_deref(), Some("529.982.247-25"));
}

#[test]
fn normalize_rejects_cpf_without_mask() {
    let city_id = Uuid::new_v4();
    let mut cmd = base_command(Some(city_id), "User", Some("52998224725"));

    let err = normalize_victim_input(&mut cmd, "ctx").unwrap_err();

    assert!(
        matches!(err, AppError::BadRequest(msg) if msg.contains("14 characters")),
        "expected 14-char length error"
    );
}

#[test]
fn normalize_rejects_cpf_with_invalid_check_digits() {
    let city_id = Uuid::new_v4();
    let mut cmd = base_command(Some(city_id), "User", Some("111.111.111-11"));

    let err = normalize_victim_input(&mut cmd, "ctx").unwrap_err();

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

    let resolved = normalize_victim_input(&mut cmd, "ctx").unwrap();

    assert_eq!(resolved, derived_city_id);
    assert_eq!(cmd.city_id, Some(derived_city_id));
}

#[test]
fn normalize_fails_when_no_city_id_and_no_addresses() {
    let mut cmd = base_command(None, "User", None);

    let err = normalize_victim_input(&mut cmd, "ctx").unwrap_err();

    assert!(
        matches!(err, AppError::BadRequest(msg) if msg.contains("city_id")),
        "expected missing city_id BadRequest"
    );
}

#[test]
fn normalize_derives_special_needs_flag_from_list() {
    let city_id = Uuid::new_v4();
    let mut cmd = base_command(Some(city_id), "User", None);
    cmd.has_special_needs = false;
    cmd.special_needs_type = Some(vec!["Motor".to_string()]);

    normalize_victim_input(&mut cmd, "ctx").unwrap();

    assert!(cmd.has_special_needs);
    assert_eq!(
        cmd.special_needs_type.as_deref(),
        Some(&["Motor".to_string()][..])
    );
}

#[test]
fn normalize_has_children_follows_children_count_presence() {
    let city_id = Uuid::new_v4();
    let mut cmd = base_command(Some(city_id), "User", None);
    cmd.children_count = Some(2);
    cmd.has_children = false;

    normalize_victim_input(&mut cmd, "ctx").unwrap();

    assert!(cmd.has_children);

    let mut cmd2 = base_command(Some(city_id), "User", None);
    cmd2.children_count = None;
    cmd2.has_children = true;

    normalize_victim_input(&mut cmd2, "ctx").unwrap();

    assert!(!cmd2.has_children);
}
