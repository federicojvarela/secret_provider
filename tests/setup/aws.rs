use std::collections::HashMap;

use crate::helpers::test_ext::SecretsProviderTestExt;
use async_trait::async_trait;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_secretsmanager::{primitives::Blob, Client};
use secrets_provider::{
    implementations::aws::AwsSecretsProvider, Decode, Secret, SecretsProvider, SecretsProviderError,
};
use serde::Deserialize;

const DEFAULT_AWS_REGION: &str = "us-west-2";

pub struct AwsTestWrapper {
    pub provider: AwsSecretsProvider,
    pub client: Client,
}

#[async_trait]
impl SecretsProvider for AwsTestWrapper {
    async fn find<T: Decode>(
        &self,
        secret_name: &str,
    ) -> Result<Option<Secret<T>>, SecretsProviderError> {
        self.provider.find(secret_name).await
    }

    async fn find_with_version<T: Decode>(
        &self,
        secret_name: &str,
        version: &str,
    ) -> Result<Option<Secret<T>>, SecretsProviderError> {
        self.provider.find_with_version(secret_name, version).await
    }

    async fn batch_find<'n, T: Decode>(
        &self,
        secret_names: &[&'n str],
    ) -> Result<HashMap<&'n str, Secret<T>>, SecretsProviderError> {
        self.provider.batch_find(secret_names).await
    }
}

#[async_trait]
impl SecretsProviderTestExt for AwsTestWrapper {
    async fn add_string_secret(&mut self, name: &str, value: &str) {
        self.create_if_not_exists(name).await;
        self.client
            .put_secret_value()
            .secret_id(name)
            .secret_string(value)
            .send()
            .await
            .unwrap();
    }

    async fn add_binary_secret(&mut self, name: &str, value: &[u8]) {
        self.create_if_not_exists(name).await;
        self.client
            .put_secret_value()
            .secret_id(name)
            .secret_binary(Blob::new(value))
            .send()
            .await
            .unwrap();
    }

    async fn list_secret_versions(&self, name: &str) -> Vec<String> {
        let secret_versions = self
            .client
            .list_secret_version_ids()
            .secret_id(name)
            .send()
            .await
            .unwrap();

        let mut versions = secret_versions.versions.unwrap();
        versions.sort_by(|a, b| {
            a.created_date
                .unwrap()
                .partial_cmp(&b.created_date.unwrap())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        versions
            .into_iter()
            .map(|e| e.version_id.unwrap())
            .collect()
    }
}

impl AwsTestWrapper {
    /// Create a RusotoSecretsProvider loading the default test configuration
    /// from files .env.test and .env.test.local.
    pub async fn load_default() -> Self {
        dotenv::from_filename(".env.test.local").ok();
        dotenv::from_filename(".env.test").ok();

        #[derive(Deserialize)]
        struct AwsProviderConfig {
            endpoint: String,
        }
        let AwsProviderConfig { endpoint } =
            envy::from_env::<AwsProviderConfig>().expect("Could not load configuration");

        let client = Client::new(
            &aws_config::defaults(BehaviorVersion::latest())
                .region(Region::new(DEFAULT_AWS_REGION.to_string()))
                .endpoint_url(&endpoint)
                .load()
                .await,
        );

        let me = Self {
            provider: AwsSecretsProvider::from(client.clone()),
            client,
        };

        me.clear_old_data().await;
        me
    }

    /// Clears any existing secret to avoid mixing data betweent tests.
    async fn clear_old_data(&self) {
        let secrets = self.client.list_secrets().send().await.unwrap();

        for secret in secrets.secret_list.unwrap().into_iter() {
            self.client
                .delete_secret()
                .force_delete_without_recovery(true)
                .secret_id(secret.name.unwrap())
                .send()
                .await
                .unwrap();
        }
    }

    async fn create_if_not_exists(&self, name: &str) {
        let _ = self
            .client
            .create_secret()
            .name(name.to_string())
            .send()
            .await;
    }
}

pub async fn load_test_provider() -> AwsTestWrapper {
    AwsTestWrapper::load_default().await
}
