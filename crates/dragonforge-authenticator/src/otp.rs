use data_encoding::BASE32_NOPAD;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::{Sha256, Sha512};

use crate::error::{AuthenticatorError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OtpAlgorithm {
    Sha1,
    Sha256,
    Sha512,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum OtpKind {
    Totp { period: u64 },
    Hotp { counter: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedCode {
    pub code: String,
    pub valid_for_seconds: Option<u64>,
    pub counter: u64,
}

pub(crate) fn normalize_secret(value: &str) -> Result<String> {
    let normalized = value
        .chars()
        .filter(|character| !character.is_whitespace() && *character != '-')
        .collect::<String>()
        .trim_end_matches('=')
        .to_ascii_uppercase();

    if normalized.len() < 16 || normalized.len() > 256 {
        return Err(AuthenticatorError::InvalidSecret);
    }

    let decoded = BASE32_NOPAD
        .decode(normalized.as_bytes())
        .map_err(|_| AuthenticatorError::InvalidSecret)?;
    if decoded.len() < 10 || decoded.len() > 128 {
        return Err(AuthenticatorError::InvalidSecret);
    }

    Ok(normalized)
}

fn decode_secret(value: &str) -> Result<Vec<u8>> {
    BASE32_NOPAD
        .decode(value.as_bytes())
        .map_err(|_| AuthenticatorError::InvalidSecret)
}

pub(crate) fn generate(
    secret_base32: &str,
    algorithm: OtpAlgorithm,
    digits: u32,
    counter: u64,
) -> Result<String> {
    if digits != 6 && digits != 8 {
        return Err(AuthenticatorError::InvalidAccount);
    }

    let secret = decode_secret(secret_base32)?;
    let message = counter.to_be_bytes();
    let digest = match algorithm {
        OtpAlgorithm::Sha1 => {
            let mut mac = Hmac::<Sha1>::new_from_slice(&secret)
                .map_err(|_| AuthenticatorError::Crypto)?;
            mac.update(&message);
            mac.finalize().into_bytes().to_vec()
        }
        OtpAlgorithm::Sha256 => {
            let mut mac = Hmac::<Sha256>::new_from_slice(&secret)
                .map_err(|_| AuthenticatorError::Crypto)?;
            mac.update(&message);
            mac.finalize().into_bytes().to_vec()
        }
        OtpAlgorithm::Sha512 => {
            let mut mac = Hmac::<Sha512>::new_from_slice(&secret)
                .map_err(|_| AuthenticatorError::Crypto)?;
            mac.update(&message);
            mac.finalize().into_bytes().to_vec()
        }
    };

    let offset = usize::from(digest[digest.len() - 1] & 0x0f);
    let slice = digest
        .get(offset..offset + 4)
        .ok_or(AuthenticatorError::Crypto)?;
    let binary = (u32::from(slice[0] & 0x7f) << 24)
        | (u32::from(slice[1]) << 16)
        | (u32::from(slice[2]) << 8)
        | u32::from(slice[3]);
    let modulus = 10_u32.pow(digits);
    Ok(format!("{:0width$}", binary % modulus, width = digits as usize))
}

pub(crate) fn generate_totp(
    secret_base32: &str,
    algorithm: OtpAlgorithm,
    digits: u32,
    period: u64,
    timestamp_seconds: u64,
) -> Result<GeneratedCode> {
    if !(15..=120).contains(&period) {
        return Err(AuthenticatorError::InvalidAccount);
    }
    let counter = timestamp_seconds / period;
    let elapsed = timestamp_seconds % period;
    Ok(GeneratedCode {
        code: generate(secret_base32, algorithm, digits, counter)?,
        valid_for_seconds: Some(period - elapsed),
        counter,
    })
}

pub(crate) fn generate_hotp(
    secret_base32: &str,
    algorithm: OtpAlgorithm,
    digits: u32,
    counter: u64,
) -> Result<GeneratedCode> {
    Ok(GeneratedCode {
        code: generate(secret_base32, algorithm, digits, counter)?,
        valid_for_seconds: None,
        counter,
    })
}

#[cfg(test)]
mod tests {
    use super::{OtpAlgorithm, generate, generate_totp, normalize_secret};

    const RFC_SECRET: &str = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";

    #[test]
    fn rfc4226_hotp_vectors_match() {
        let expected = [
            "755224", "287082", "359152", "969429", "338314",
            "254676", "287922", "162583", "399871", "520489",
        ];
        for (counter, expected_code) in expected.iter().enumerate() {
            assert_eq!(
                generate(RFC_SECRET, OtpAlgorithm::Sha1, 6, counter as u64).expect("hotp"),
                *expected_code
            );
        }
    }

    #[test]
    fn rfc6238_sha1_vector_matches() {
        let generated =
            generate_totp(RFC_SECRET, OtpAlgorithm::Sha1, 8, 30, 59).expect("totp");
        assert_eq!(generated.code, "94287082");
        assert_eq!(generated.valid_for_seconds, Some(1));
    }

    #[test]
    fn rfc6238_sha256_and_sha512_vectors_match() {
        const SHA256_SECRET: &str =
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZA";
        const SHA512_SECRET: &str =
            "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQGEZDGNA";
        assert_eq!(
            generate_totp(SHA256_SECRET, OtpAlgorithm::Sha256, 8, 30, 59)
                .expect("sha256")
                .code,
            "46119246"
        );
        assert_eq!(
            generate_totp(SHA512_SECRET, OtpAlgorithm::Sha512, 8, 30, 59)
                .expect("sha512")
                .code,
            "90693936"
        );
    }

    #[test]
    fn secret_normalization_accepts_spacing_and_padding() {
        assert_eq!(
            normalize_secret("GEZD GNBV-GY3TQOJQ====").expect("secret"),
            "GEZDGNBVGY3TQOJQ"
        );
    }
}
