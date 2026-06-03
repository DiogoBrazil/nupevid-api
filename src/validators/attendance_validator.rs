use chrono::NaiveDate;

use crate::core::application_error::ApplicationError as AppError;
use crate::core::entities::attendance_offenders::ViolenceAggravator;

pub struct AttendanceValidator;

impl AttendanceValidator {
    pub fn validate_attendance_date_not_future(
        attendance_date: NaiveDate,
        today: NaiveDate,
    ) -> Result<(), AppError> {
        if attendance_date > today {
            return Err(AppError::BadRequest(
                "'attendance_date' cannot be in the future".to_string(),
            ));
        }
        Ok(())
    }

    pub fn validate_violence_aggravator(
        aggravator: &Option<ViolenceAggravator>,
        other: &Option<String>,
    ) -> Result<(), AppError> {
        let other_is_present = other.as_ref().is_some_and(|value| !value.trim().is_empty());

        match aggravator {
            Some(ViolenceAggravator::Other) => {
                if !other_is_present {
                    return Err(AppError::BadRequest(
                        "'violence_aggravator_other' is required when violence_aggravator is 'Other'"
                            .to_string(),
                    ));
                }
            }
            _ => {
                if other_is_present {
                    return Err(AppError::BadRequest(
                        "'violence_aggravator_other' is only allowed when violence_aggravator is 'Other'"
                            .to_string(),
                    ));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 6, 2).unwrap()
    }

    #[test]
    fn date_today_is_ok() {
        let result = AttendanceValidator::validate_attendance_date_not_future(today(), today());
        assert!(result.is_ok());
    }

    #[test]
    fn date_in_the_past_is_ok() {
        let past = today() - Duration::days(30);
        let result = AttendanceValidator::validate_attendance_date_not_future(past, today());
        assert!(result.is_ok());
    }

    #[test]
    fn date_in_the_future_is_rejected() {
        let future = today() + Duration::days(1);
        let result = AttendanceValidator::validate_attendance_date_not_future(future, today());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("'attendance_date' cannot be in the future")
        );
    }

    #[test]
    fn aggravator_other_without_text_is_rejected() {
        let result = AttendanceValidator::validate_violence_aggravator(
            &Some(ViolenceAggravator::Other),
            &None,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("is required"));
    }

    #[test]
    fn aggravator_other_with_blank_text_is_rejected() {
        let result = AttendanceValidator::validate_violence_aggravator(
            &Some(ViolenceAggravator::Other),
            &Some("   ".to_string()),
        );
        assert!(result.is_err());
    }

    #[test]
    fn aggravator_other_with_text_is_ok() {
        let result = AttendanceValidator::validate_violence_aggravator(
            &Some(ViolenceAggravator::Other),
            &Some("Histórico de ameaças".to_string()),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn non_other_aggravator_with_text_is_rejected() {
        let result = AttendanceValidator::validate_violence_aggravator(
            &Some(ViolenceAggravator::AlcoholUse),
            &Some("texto indevido".to_string()),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("is only allowed"));
    }

    #[test]
    fn non_other_aggravator_without_text_is_ok() {
        let result = AttendanceValidator::validate_violence_aggravator(
            &Some(ViolenceAggravator::DrugUse),
            &None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn none_aggravator_without_text_is_ok() {
        let result = AttendanceValidator::validate_violence_aggravator(&None, &None);
        assert!(result.is_ok());
    }

    #[test]
    fn none_aggravator_with_text_is_rejected() {
        let result = AttendanceValidator::validate_violence_aggravator(
            &None,
            &Some("texto".to_string()),
        );
        assert!(result.is_err());
    }
}
