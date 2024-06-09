use async_trait::async_trait;

/// This trait defines some test extensions to seed a secrets provider before
/// each test.
#[async_trait]
pub trait SecretsProviderTestExt {
    /// Inserts or replaces a string secret in the secret manager.
    ///
    /// Returns the current version of the inserted secret.
    async fn add_string_secret(&mut self, name: &str, value: &str);

    /// Inserts or replaces a binary secret in the secret manager.
    ///
    /// Returns the current version of the inserted secret.
    async fn add_binary_secret(&mut self, name: &str, value: &[u8]);

    /// Lists the existing versions of a given secret, sorted by creation
    /// date. Each call to `add_string_secret` will create a new version
    /// for a given secret name.
    async fn list_secret_versions(&self, name: &str) -> Vec<String>;
}
