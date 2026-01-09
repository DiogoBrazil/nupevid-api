use crate::utils::errors::AppError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CpfError {
    InvalidLength,
    InvalidFormat,
    InvalidDigits,
}

pub fn validate_cpf_masked(cpf: &str) -> Result<String, CpfError> {
    let cpf = cpf.trim();

    if cpf.len() != 14 {
        return Err(CpfError::InvalidLength);
    }

    let b = cpf.as_bytes();
    if b[3] != b'.' || b[7] != b'.' || b[11] != b'-' {
        return Err(CpfError::InvalidFormat);
    }

    let digit_positions = [0usize, 1, 2, 4, 5, 6, 8, 9, 10, 12, 13];
    if !digit_positions.iter().all(|&i| b[i].is_ascii_digit()) {
        return Err(CpfError::InvalidFormat);
    }

    let mut numbers = [0u8; 11];
    let mut idx = 0usize;
    for ch in cpf.chars() {
        if ch.is_ascii_digit() {
            if idx >= numbers.len() {
                return Err(CpfError::InvalidFormat);
            }
            numbers[idx] = (ch as u8) - b'0';
            idx += 1;
        }
    }

    if numbers.iter().all(|&d| d == numbers[0]) {
        return Err(CpfError::InvalidDigits);
    }

    let sum1: u32 = (0..9).map(|i| numbers[i] as u32 * (10 - i as u32)).sum();
    let mut check1 = (sum1 * 10) % 11;
    if check1 == 10 {
        check1 = 0;
    }
    if check1 as u8 != numbers[9] {
        return Err(CpfError::InvalidDigits);
    }

    let sum2: u32 = (0..10).map(|i| numbers[i] as u32 * (11 - i as u32)).sum();
    let mut check2 = (sum2 * 10) % 11;
    if check2 == 10 {
        check2 = 0;
    }
    if check2 as u8 != numbers[10] {
        return Err(CpfError::InvalidDigits);
    }

    Ok(cpf.to_string())
}

pub fn validate_cpf(cpf: &str, error_context: &str) -> Result<String, AppError> {
    match validate_cpf_masked(cpf) {
        Ok(normalized) => Ok(normalized),
        Err(CpfError::InvalidLength) => Err(AppError::BadRequest(format!(
            "{}: cpf must be 14 characters in the format 000.000.000-00",
            error_context
        ))),
        Err(CpfError::InvalidFormat) => Err(AppError::BadRequest(format!(
            "{}: cpf must match the format 000.000.000-00",
            error_context
        ))),
        Err(CpfError::InvalidDigits) => Err(AppError::BadRequest(format!(
            "{}: cpf has invalid check digits",
            error_context
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_cpf_masked_success() {
        let result = validate_cpf_masked("529.982.247-25");
        assert!(result.is_ok());
    }

    #[test]
    fn validate_cpf_masked_invalid_length() {
        let result = validate_cpf_masked("529.982.247-2");
        assert_eq!(result.unwrap_err(), CpfError::InvalidLength);
    }

    #[test]
    fn validate_cpf_masked_invalid_format() {
        let result = validate_cpf_masked("529-982-247-25");
        assert_eq!(result.unwrap_err(), CpfError::InvalidFormat);
    }

    #[test]
    fn validate_cpf_masked_invalid_digits() {
        let result = validate_cpf_masked("111.111.111-11");
        assert_eq!(result.unwrap_err(), CpfError::InvalidDigits);
    }
}
