use async_trait::async_trait;
use secrets_provider::implementations::memory::MemorySecretsProvider;

use crate::helpers::test_ext::SecretsProviderTestExt;

#[async_trait]
impl SecretsProviderTestExt for MemorySecretsProvider {
    async fn add_string_secret(&mut self, name: &str, value: &str) {
        MemorySecretsProvider::add_string_secret(self, name.into(), value.into());
    }

    async fn add_binary_secret(&mut self, name: &str, value: &[u8]) {
        MemorySecretsProvider::add_binary_secret(self, name.into(), value.into());
    }

    async fn list_secret_versions(&self, name: &str) -> Vec<String> {
        self.list_secret_version_ids(name).unwrap()
    }
}

pub fn load_test_provider() -> MemorySecretsProvider {
    MemorySecretsProvider::default()
}
