use std::fmt::{Debug, Display};

use crate::errors::SecretsProviderError;
use crate::Result;

/// Contains the secret data.
///
/// We use this enum to know which datatype is the secret and to corretly downcast it when it is
/// needed. The caller must know the secret's datatype before use it.
pub enum SecretData {
    Str(String),
    Bytes(Vec<u8>),
}

/// Structure containing a secret retrieved from a secret manager.
///
/// This structure also holds some metadata (such as the version and name). The secret can not be
/// accessed directly. To access it you shuld use the [reveal()][crate::secret::Secret::reveal]
/// function.
pub struct Secret<T> {
    /// Name or key of the secret
    pub name: String,

    /// Secret's version
    pub version: String,

    /// Secret itself
    pub(crate) secret: T,
}

impl<T> Secret<T> {
    /// Revels the secret, detroying the self.
    ///
    /// This example uses the `memory` feature.
    #[cfg_attr(not(feature = "memory"), doc = "```ignore")]
    /// ```rust,no_run
    /// use secrets_provider::{SecretsProvider, implementations::memory::MemorySecretsProvider};
    ///
    /// fn get_secrets_provider() -> impl SecretsProvider {
    ///     MemorySecretsProvider::new()
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let secrets_provider = get_secrets_provider();
    ///
    ///     let secret = secrets_provider
    ///         .find::<String>("master_key_of_everything")
    ///         .await
    ///         .expect("There was an error getting the Master Key of Everything")
    ///         .expect("Secret not found");
    ///
    ///     let secret_key = secret.reveal();
    /// }
    /// ```
    pub fn reveal(self) -> T {
        self.secret
    }
}

// We use this custom implementation of Display to prevent accidental secret leaking through
// printing
impl<T> Display for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"{{ name: {}, version: {} }}"#,
            &self.name, self.version
        )
    }
}

// We use this custom implementation of Debug to prevent accidental secret leaking through
// printing
impl<T> Debug for Secret<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Secret")
            .field("name", &self.name)
            .field("version", &self.version)
            .field("secret", &"*****")
            .finish()
    }
}

/// Trait used to cast a secret retrieved from a manager to its correct type. To support a new type
/// T, T must implement Decode, and add a variant for T in the
/// [SecretData](crate::secret::SecretData] enum.
pub trait Decode: Send {
    /// Tries to cast the [SecretData](SecretData) into its real datatype.
    ///
    /// # Arguments
    ///
    /// * `secret_name` - A string that contains the secret name.
    /// * `secret_data` - Contains the information about the type of the secret and the secret itself.
    fn decode(secret_name: &str, secret_data: SecretData) -> Result<Self>
    where
        Self: Sized;
}

impl Decode for String {
    fn decode(secret_name: &str, secret_data: SecretData) -> Result<Self> {
        match secret_data {
            SecretData::Str(s) => Ok(s),
            _ => Err(SecretsProviderError::InvalidType(secret_name.to_string())),
        }
    }
}

impl Decode for Vec<u8> {
    fn decode(secret_name: &str, secret_data: SecretData) -> Result<Self> {
        match secret_data {
            SecretData::Bytes(b) => Ok(b),
            _ => Err(SecretsProviderError::InvalidType(secret_name.to_string())),
        }
    }
}
