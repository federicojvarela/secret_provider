//! Amazon Web Services Secret Provider implementation using AWS' official SDK
//!
//! In order to connect to the real AWS, the following
//!
//! * `AWS_WEB_IDENTITY_TOKEN_FILE` - Path to the web identity token file.
//! * `AWS_ROLE_ARN` - ARN of the role to assume.
//! * `AWS_ROLE_SESSION_NAME` - **(optional)** name applied to the assume-role session.
//!
//! For more information:
//! `<https://docs.aws.amazon.com/sdk-for-rust/latest/dg/environment-variables.html>`
use async_trait::async_trait;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_secretsmanager::error::SdkError;
use aws_sdk_secretsmanager::operation::get_secret_value::{
    GetSecretValueError, GetSecretValueOutput,
};
use aws_sdk_secretsmanager::Client;

use crate::errors::SecretsProviderError;
use crate::secret::{Decode, Secret, SecretData};
use crate::{Result, SecretsProvider};

/// Amazon Web Services Secrets Provider implementation.
#[derive(Clone)]
pub struct AwsSecretsProvider {
    client: Client,
}

impl AwsSecretsProvider {
    /// Creates a new Secrets Provider for Amazon Web Services.
    ///
    /// # Arguments
    ///
    /// * `region` - String representing the AWS Region. Must be formatted with all lowercases
    /// letters and hyphens. For example: `us-west-2`.
    pub async fn new(region: String) -> Self {
        Self {
            client: Client::new(
                &aws_config::defaults(BehaviorVersion::latest())
                    .region(Region::new(region))
                    .load()
                    .await,
            ),
        }
    }

    /// Creates a new Secrets Provider for Amazon Web Services at a given URL. This method
    /// can be used to connect to AWS emulators like Localstack.
    ///
    /// # Arguments
    ///
    /// * `region` - String representing the AWS Region. Must be formatted with all lowercases
    /// letters and hyphens. For example: `us-west-2`.
    /// * `endpoint_url` - URL of the AWS emulator. Example: `http://localhost:4566`.
    pub async fn new_at_endpoint(region: &str, endpoint_url: &str) -> Self {
        Self {
            client: Client::new(
                &aws_config::defaults(BehaviorVersion::latest())
                    .region(Region::new(region.to_string()))
                    .endpoint_url(endpoint_url)
                    .load()
                    .await,
            ),
        }
    }

    fn parse_response<T: Decode>(
        secret_id: &str,
        response: GetSecretValueOutput,
    ) -> Result<Option<Secret<T>>> {
        let GetSecretValueOutput {
            version_id,
            name,
            secret_string,
            secret_binary,
            ..
        } = response;

        let name = name.unwrap_or_else(|| secret_id.to_string());
        Ok(Some(Secret {
            version: version_id.unwrap_or_else(|| "unknown".to_string()),
            secret: T::decode(
                &name,
                if let Some(d) = secret_string {
                    Ok(SecretData::Str(d))
                } else if let Some(d) = secret_binary {
                    Ok(SecretData::Bytes(d.into_inner()))
                } else {
                    Err(SecretsProviderError::UnknownType(name.clone()))
                }?,
            )?,
            name,
        }))
    }

    async fn find_secret<T: Decode>(
        &self,
        name: &str,
        version: Option<&str>,
    ) -> Result<Option<Secret<T>>> {
        let mut request = self.client.get_secret_value().secret_id(name);
        if let Some(version) = version {
            request = request.version_id(version);
        }

        match request.send().await {
            Ok(response) => Self::parse_response(name, response),
            Err(SdkError::ServiceError(e)) => match e.err() {
                GetSecretValueError::ResourceNotFoundException(_) => Ok(None),
                other => Err(SecretsProviderError::ProviderFailed(other.to_string())),
            },
            Err(other) => Err(SecretsProviderError::ProviderFailed(other.to_string())),
        }
    }
}

#[async_trait]
impl SecretsProvider for AwsSecretsProvider {
    async fn find<T: Decode>(&self, key_name: &str) -> Result<Option<Secret<T>>> {
        self.find_secret(key_name, None).await
    }

    async fn find_with_version<T: Decode>(
        &self,
        key_name: &str,
        version: &str,
    ) -> Result<Option<Secret<T>>> {
        self.find_secret(key_name, Some(version)).await
    }

    // NOTE: The official SDK provides the `batch_get_secret_value` method which would
    // be a more efficient implementation of the `batch_find` method. However, it's
    // still too recent to the point it's lacking support in localstack.
    //
    // We'll override the default implementation once we know localstack supports it so
    // it doesn't block the development process / integration testing / pipelines.
}

impl From<Client> for AwsSecretsProvider {
    fn from(client: Client) -> Self {
        Self { client }
    }
}
