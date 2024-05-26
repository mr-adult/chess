use std::str::FromStr;

use hmac::{Hmac, Mac};
use sha2::Sha512;

pub struct EncryptContent {
    pub content_clear_text: String,
    pub salt_clear_text: String,
}

#[derive(Debug)]
pub enum CryptErr {
    KeyFailHmac,
    PasswordMismatch,
    InvalidPasswordString,
}

/// This enum is a marker for the encryption scheme to allow
/// it to be changed in the future
pub(crate) struct EncryptedPassword {
    scheme: PasswordEncryptionScheme,
    encrypted_password: String,
}

impl EncryptedPassword {
    pub(crate) fn encrypt(
        key: &[u8],
        encrypt_content: &EncryptContent,
    ) -> Result<EncryptedPassword, CryptErr> {
        Self::encrypt_with_scheme(
            key,
            encrypt_content,
            PasswordEncryptionScheme::Base64EncodedSha256,
        )
    }

    fn encrypt_with_scheme(
        key: &[u8],
        encrypt_content: &EncryptContent,
        scheme: PasswordEncryptionScheme,
    ) -> Result<EncryptedPassword, CryptErr> {
        match scheme {
            PasswordEncryptionScheme::Base64EncodedSha256 => {
                Self::encrypt_base_64_url_encoded_sha_256(key, encrypt_content, scheme)
            }
        }
    }

    fn encrypt_base_64_url_encoded_sha_256(
        key: &[u8],
        encrypt_content: &EncryptContent,
        scheme: PasswordEncryptionScheme,
    ) -> Result<EncryptedPassword, CryptErr> {
        let EncryptContent {
            content_clear_text,
            salt_clear_text,
        } = encrypt_content;

        let mut hmac_sha512 =
            Hmac::<Sha512>::new_from_slice(key).map_err(|_| CryptErr::KeyFailHmac)?;

        hmac_sha512.update(content_clear_text.as_bytes());
        hmac_sha512.update(salt_clear_text.as_bytes());

        let hmac_result = hmac_sha512.finalize();
        let result_bytes = hmac_result.into_bytes();

        let result = base64_url::encode(&result_bytes);
        return Ok(EncryptedPassword {
            scheme: PasswordEncryptionScheme::Base64EncodedSha256,
            encrypted_password: result,
        });
    }

    fn validate_matches(&self, key: &[u8], password: &EncryptContent) -> Result<(), CryptErr> {
        let encrypted = Self::encrypt_with_scheme(key, password, self.scheme)?;
        if self.encrypted_password == encrypted.encrypted_password {
            return Ok(());
        } else {
            return Err(CryptErr::PasswordMismatch);
        }
    }
}

impl ToString for EncryptedPassword {
    fn to_string(&self) -> String {
        format!("{}{}", self.scheme.to_string(), self.encrypted_password)
    }
}

impl FromStr for EncryptedPassword {
    type Err = CryptErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("#001#") {
            return Ok(EncryptedPassword {
                scheme: PasswordEncryptionScheme::Base64EncodedSha256,
                encrypted_password: s[5..].to_string(),
            });
        } else {
            return Err(CryptErr::InvalidPasswordString);
        }
    }
}

#[derive(Clone, Copy)]
pub enum PasswordEncryptionScheme {
    Base64EncodedSha256,
}

impl PasswordEncryptionScheme {
    const fn code(&self) -> u16 {
        match self {
            Self::Base64EncodedSha256 => 1,
        }
    }
}

impl ToString for PasswordEncryptionScheme {
    fn to_string(&self) -> String {
        format!("#{:03}#", self.code())
    }
}

pub fn encrypt_password(encrypt_content: &EncryptContent) -> Result<EncryptedPassword, CryptErr> {
    let key = b"My totally secret key";
    let encrypted = EncryptedPassword::encrypt(key, encrypt_content)?;
    return Ok(encrypted);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ensure consistency in our encryption algorithm
    #[test]
    fn test_encrypt_into_base_64_url() {
        // Randomly generated using the rand crate
        let test_key = [
            18, 137, 164, 20, 17, 81, 23, 195, 248, 135, 245, 226, 72, 93, 160, 58, 5, 5, 154, 113,
            153, 130, 27, 50, 21, 78, 165, 52, 139, 154, 3, 61, 80, 33, 184, 12, 40, 184, 0, 81,
            61, 202, 240, 29, 41, 35, 237, 206, 64, 59, 220, 159, 240, 153, 113, 60, 122, 245, 173,
            71, 134, 254, 254, 14,
        ];

        let result = EncryptedPassword::encrypt_with_scheme(
            &test_key,
            &EncryptContent {
                content_clear_text: "hello world".to_string(),
                salt_clear_text: "some salt".to_string(),
            },
            PasswordEncryptionScheme::Base64EncodedSha256,
        )
        .unwrap()
        .encrypted_password;

        assert_eq!(
            "lebD8CkbHCR3EKjGw7wI72_MEg2CCfinY6dTyonCSZSGVhnBFOI6KlxoYWt00Ni40ljgE4qfERTBeemIF8ZRdw", 
            result);
    }

    #[test]
    fn encryption_scheme_one_builds_one() {
        assert_eq!(1, PasswordEncryptionScheme::Base64EncodedSha256.code());
        assert_eq!(
            "#001#",
            PasswordEncryptionScheme::Base64EncodedSha256.to_string()
        )
    }
}
