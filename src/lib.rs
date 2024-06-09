//! # Secrets provider
//!
//! The objetive of this crate is to abstract away secret managment solutions and have a common
//! interface for all of them.
//!
//! ## Supported types
//! Right now two datatypes are supported for secrets:
//! - string: Represented with the [String](std::string::String) type.
//! - binary: Represented with the [Vec<u8>](std::vec::Vec) type.
//!
//! This means that you have to explicitly type the function
//! [get_secret](crate::SecretsProvider::get_secret) with turbofish (`::<T>`) or use it
//! in a context where the type can be inferred.
mod errors;
pub mod implementations;
mod secret;

use std::collections::HashMap;

use async_trait::async_trait;
pub use errors::SecretsProviderError;
pub use secret::{Decode, Secret};

type Result<T> = std::result::Result<T, SecretsProviderError>;

/// Secrets provider implementations interface.
#[async_trait]
pub trait SecretsProvider {
    /// Retrieves a secret from the secret provider.
    ///
    /// We are using async_trait, and that is why the signature of this function is so strange.
    /// The pretty-printed signature for this functions is:
    /// ```rust,ignore
    /// async fn find<T: Decode>(
    ///     &self,
    ///     secret_name: &str,
    ///     version: Option<String>,
    /// ) -> Result<Option<Secret<T>>>;
    /// ```
    ///
    /// # Arguments
    ///
    /// * `secret_name` - A string that contains the secret name.
    /// * `version` - The secret's version (if there is one).
    ///
    /// # Example
    ///
    /// This method must be used with the turbofish syntax (`::<T>`) or in a context where the
    /// compiler can infer the type of the secret we are trying to retrieve.
    ///
    /// This example uses the `memory` feature
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
    ///     let string_secret = secrets_provider
    ///         .find::<String>("master_key_of_everything")
    ///         .await
    ///         .expect("There was an error getting the Master Key of Everything");
    ///     if let Some(secret) = string_secret {
    ///         println!("The Master Key of Everything is: {}", secret.reveal());
    ///     } else {
    ///         println!("Sadly there is no Master Key of Everything :(");
    ///     }
    /// }
    /// ```
    async fn find<T: Decode>(&self, secret_name: &str) -> Result<Option<Secret<T>>>;

    /// Retrieves a specific version of a secret from the secret provider.
    ///
    /// We are using async_trait, and that is why the signature of this function is so strange.
    /// The pretty-printed signature for this functions is:
    /// ```rust,ignore
    /// async fn find_with_version<T: Decode>(
    ///     &self,
    ///     secret_name: &str,
    ///     version: &str,
    /// ) -> Result<Option<Secret<T>>>;
    /// ```
    ///
    /// # Arguments
    ///
    /// * `secret_name` - A string that contains the secret name.
    /// * `version` - The secret's version to retrieve.
    ///
    /// # Example
    ///
    /// This method must be used with the turbofish syntax (`::<T>`) or in a context where the
    /// compiler can infer the type of the secret we are trying to retrieve.
    ///
    /// This example uses the `memory` feature
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
    ///     let string_secret = secrets_provider
    ///         .find_with_version::<String>("master_key_of_everything", "Taylor's")
    ///         .await
    ///         .expect("There was an error getting the Master Key of Everything");
    ///     if let Some(secret) = string_secret {
    ///         println!("The Master Key of Everything is: {} (version=Taylor's)", secret.reveal());
    ///     } else {
    ///         println!("Sadly there is no Master Key of Everything (Taylor's version) :(");
    ///     }
    /// }
    /// ```
    async fn find_with_version<T: Decode>(
        &self,
        secret_name: &str,
        version: &str,
    ) -> Result<Option<Secret<T>>>;

    /// Retrieves a group of secrets in a single request from the secrets provider.
    ///
    /// The last version of each secret will be retrieved. This method will try to
    /// retrieve as many secrets as you specify in the secret_names array. Secrets
    /// that cannot be found or retrieved (because they don't exist or you don't
    /// have permissions to retrieve them) will not cause the method to return an
    /// Err but will not be listed in the resulting HashMap. The Err variant will be
    /// returned in cases where the complete request fails or for some other reason the
    /// secrets provider is unreacheable.
    ///
    /// A naive default implementation of this method is provided that simply calls
    /// `Self::find` in a loop. Implementations may override this implementation with
    /// a more efficient version if they support it.
    ///
    /// Note that some implementations may impose a limit on how many secrets can be
    /// retrieved at once. AWS, for example, limits this to 20 secrets.
    ///
    /// We are using async_trait, and that is why the signature of this function is so strange.
    /// The pretty-printed signature for this functions is:
    /// ```rust,ignore
    /// async fn batch_find<T: Decode>(
    ///     &self,
    ///     secret_names: &[&str],
    /// ) -> Result<HashMap<String, Secret<T>>>;
    /// ```
    ///
    /// # Arguments
    ///
    /// * `secret_names` - List of secret names that will be retrieved
    ///
    /// # Example
    ///
    /// This method must be used with the turbofish syntax (`::<T>`) or in a context where the
    /// compiler can infer the type of the secret we are trying to retrieve.
    ///
    /// This example uses the `memory` feature
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
    ///     let secrets = secrets_provider
    ///         .batch_find::<String>(&["secret_1", "secret_2", "secret_not_found"])
    ///         .await
    ///         .expect("There was an error retrieving secrets");
    ///     
    ///     assert!(secrets.get("secret_1").is_some());
    ///     assert!(secrets.get("secret_2").is_some());
    ///     assert!(secrets.get("secret_not_found").is_none());
    /// }
    /// ```
    async fn batch_find<'n, T: Decode>(
        &self,
        secret_names: &[&'n str],
    ) -> Result<HashMap<&'n str, Secret<T>>> {
        let mut retrieved = HashMap::new();
        for name in secret_names {
            if let Some(secret) = self.find(name).await? {
                retrieved.insert(*name, secret);
            }
        }

        Ok(retrieved)
    }
}
