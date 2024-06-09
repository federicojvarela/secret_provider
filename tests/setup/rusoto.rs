use std::collections::HashMap;

use crate::helpers::test_ext::SecretsProviderTestExt;
use async_trait::async_trait;
use rusoto_core::Region;
use rusoto_secretsmanager::{
    CreateSecretRequest, DeleteSecretRequest, ListSecretVersionIdsRequest, ListSecretsRequest,
    PutSecretValueRequest, SecretsManager, SecretsManagerClient,
};
use secrets_provider::{
    implementations::rusoto::AwsSecretsProvider, Decode, Secret, SecretsProvider,
    SecretsProviderError,
};
use serde::Deserialize;

const DEFAULT_AWS_REGION: &str = "us-west-2";

pub struct RusotoTestWrapper {
    pub provider: AwsSecretsProvider,
    pub client: SecretsManagerClient,
}

#[async_trait]
impl SecretsProvider for RusotoTestWrapper {
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
impl SecretsProviderTestExt for RusotoTestWrapper {
    async fn add_string_secret(&mut self, name: &str, value: &str) {
        self.create_if_not_exists(name).await;
        self.client
            .put_secret_value(PutSecretValueRequest {
                secret_id: name.into(),
                secret_string: Some(value.into()),
                client_request_token: Some(uuid::Uuid::new_v4().to_string()),
                ..Default::default()
            })
            .await
            .unwrap();
    }

    async fn add_binary_secret(&mut self, name: &str, value: &[u8]) {
        self.create_if_not_exists(name).await;
        self.client
            .put_secret_value(PutSecretValueRequest {
                secret_id: name.into(),
                secret_binary: Some(value.to_vec().into()),
                client_request_token: Some(uuid::Uuid::new_v4().to_string()),
                ..Default::default()
            })
            .await
            .unwrap();
    }

    async fn list_secret_versions(&self, name: &str) -> Vec<String> {
        let secret_versions = self
            .client
            .list_secret_version_ids(ListSecretVersionIdsRequest {
                secret_id: name.into(),
                ..Default::default()
            })
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

impl RusotoTestWrapper {
    /// Create a RusotoSecretsProvider loading the default test configuration
    /// from files .env.test and .env.test.local.
    pub async fn load_default() -> Self {
        dotenv::from_filename(".env.test.local").ok();
        dotenv::from_filename(".env.test").ok();

        #[derive(Deserialize)]
        struct RusotoProviderConfig {
            endpoint: String,
        }
        let config =
            envy::from_env::<RusotoProviderConfig>().expect("Could not load configuration");

        let client = SecretsManagerClient::new(Region::Custom {
            name: DEFAULT_AWS_REGION.to_string(),
            endpoint: config.endpoint,
        });

        let me = Self {
            provider: AwsSecretsProvider::new_with_secrets_manager_client(client.clone()).unwrap(),
            client,
        };

        me.clear_old_data().await;
        me
    }

    /// Clears any existing secret to avoid mixing data betweent tests.
    async fn clear_old_data(&self) {
        let secrets = self
            .client
            .list_secrets(ListSecretsRequest::default())
            .await
            .unwrap();

        for secret in secrets.secret_list.unwrap().into_iter() {
            self.client
                .delete_secret(DeleteSecretRequest {
                    force_delete_without_recovery: Some(true),
                    secret_id: secret.name.unwrap(),
                    ..Default::default()
                })
                .await
                .unwrap();
        }
    }

    async fn create_if_not_exists(&self, name: &str) {
        let _ = self
            .client
            .create_secret(CreateSecretRequest {
                name: name.into(),
                client_request_token: Some(uuid::Uuid::new_v4().to_string()),
                ..Default::default()
            })
            .await;
    }
}

pub async fn load_test_provider() -> RusotoTestWrapper {
    RusotoTestWrapper::load_default().await
}
