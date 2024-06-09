//! Amazon Web Services Secret Provider implementation
//!
//! In order to connect to the real AWS, the following
//!
//! * `AWS_WEB_IDENTITY_TOKEN_FILE` - Path to the web identity token file.
//! * `AWS_ROLE_ARN` - ARN of the role to assume.
//! * `AWS_ROLE_SESSION_NAME` - **(optional)** name applied to the assume-role session.
//!
//! For more information:
//! `<https://docs.rs/rusoto_sts/0.48.0/rusoto_sts/struct.WebIdentityProvider.html#method.from_k8s_env>`
use async_trait::async_trait;
use rusoto_core::Region;
use rusoto_credential::AutoRefreshingProvider;
use rusoto_secretsmanager::{
    GetSecretValueError, GetSecretValueRequest, GetSecretValueResponse, SecretsManager,
    SecretsManagerClient,
};
use rusoto_sts::WebIdentityProvider;
use std::str::FromStr;

use crate::errors::SecretsProviderError;
use crate::secret::{Decode, Secret, SecretData};
use crate::{Result, SecretsProvider};

/// Amazon Web Services Secrets Provider builder.
pub struct AwsSecretsProviderBuilder {
    /// AWS Region where is located the Secret Manager.
    region: String,

    /// Endpoint of the service.
    endpoint: Option<String>,
}

impl AwsSecretsProviderBuilder {
    /// Creates a new Amazon Web Services Secrets Provider builder.
    ///
    /// # Arguments
    ///
    /// * `region` - String represeting the AWS Region. Must be formatted with all lowercases
    /// letters and hyphens. For example: `us-west-2`.
    pub fn new(region: String) -> Self {
        Self {
            region,
            endpoint: None,
        }
    }

    /// Overrides the connection endpoint.
    ///
    /// This is usually used for testing purposes (for example, using the localstack endpoit).
    ///
    /// # Arguments
    ///
    /// * `endpoint` - String represeting the endpoint. For example: `http://127.0.0.1:4566`.
    pub fn endpoint_override(mut self, endpoint: String) -> Self {
        self.endpoint = Some(endpoint);
        self
    }

    /// Builds a [AwsSecretsProvider](crate::implementations::AwsSecretsProvider).
    pub fn build(self) -> Result<AwsSecretsProvider> {
        AwsSecretsProvider::new(self.region, self.endpoint)
    }
}

/// Amazon Web Services Secrets Provider implementation.
#[derive(Clone)]
pub struct AwsSecretsProvider {
    secrets_manager_client: SecretsManagerClient,
}

impl AwsSecretsProvider {
    /// Creates a new Secrets Provider for Amazon Web Services.
    ///
    /// # Arguments
    ///
    /// * `region` - String represeting the AWS Region. Must be formatted with all lowercases
    /// letters and hyphens. For example: `us-west-2`.
    /// * `endpoint` - String represeting the endpoint. For example: `http://127.0.0.1:4566`. This
    /// string is Optional because overriding the endpoint usually means the Secrets Provider will
    /// be used in a test environment.
    fn new(region: String, endpoint: Option<String>) -> Result<Self> {
        let secrets_manager_client = if let Some(ep) = endpoint {
            SecretsManagerClient::new(Region::Custom {
                name: region,
                endpoint: ep,
            })
        } else {
            let region = Region::from_str(&region).map_err(|e| {
                SecretsProviderError::Initialization(format!(
                    r#"Unable to parse AWS region "{}": {}"#,
                    &region, e
                ))
            })?;

            let request_dispatcher = rusoto_core::request::HttpClient::new().map_err(|e| {
                SecretsProviderError::Initialization(format!(
                    "Unable to build Rusoto HTTP Client: {}",
                    e
                ))
            })?;

            // Create a WebIdentityProvider from the following environment variables:
            // - AWS_WEB_IDENTITY_TOKEN_FILE path to the web identity token file.
            // - AWS_ROLE_ARN ARN of the role to assume.
            // - AWS_ROLE_SESSION_NAME (optional) name applied to the assume-role session.
            // https://docs.rs/rusoto_sts/0.45.0/rusoto_sts/struct.WebIdentityProvider.html#method.from_k8s_env
            let credentials_provider =
                AutoRefreshingProvider::new(WebIdentityProvider::from_k8s_env()).map_err(|e| {
                    SecretsProviderError::Initialization(format!(
                        "Unable to construct the credentials provider from k8s environment: {}",
                        e
                    ))
                })?;

            SecretsManagerClient::new_with(request_dispatcher, credentials_provider, region)
        };

        Self::new_with_secrets_manager_client(secrets_manager_client)
    }

    /// Creates a new Secrets Provider for Amazon Web Services using an instance of a `SecretsManagerClient`.
    ///
    /// # Arguments
    ///
    /// * `secrets_manager_client` - Instance of `SecretsManagerClient`. Useful for reusing credentials providers.
    pub fn new_with_secrets_manager_client(
        secrets_manager_client: SecretsManagerClient,
    ) -> Result<Self> {
        Ok(AwsSecretsProvider {
            secrets_manager_client,
        })
    }

    /// Processes an AWS Secret Manager response and creates a
    /// [SecretData](crate::secret::SecretData) value from the response.
    ///
    /// # Arguments
    ///
    /// * `secret_id` - A string that contains the secret name.
    /// * `response` - Response from AWS Secret Manager Client.
    fn parse_response<T: Decode>(
        secret_id: &str,
        response: GetSecretValueResponse,
    ) -> Result<Secret<T>> {
        let GetSecretValueResponse {
            version_id,
            name,
            secret_string,
            secret_binary,
            ..
        } = response;

        let name = name.unwrap_or_else(|| secret_id.to_string());
        Ok(Secret {
            version: version_id.unwrap_or_else(|| "unknown".to_string()),
            secret: T::decode(
                &name,
                if let Some(d) = secret_string {
                    Ok(SecretData::Str(d))
                } else if let Some(d) = secret_binary {
                    Ok(SecretData::Bytes(d.to_vec()))
                } else {
                    Err(SecretsProviderError::UnknownType(name.clone()))
                }?,
            )?,
            name,
        })
    }

    async fn find_secret<T: Decode>(
        &self,
        name: &str,
        version: Option<&str>,
    ) -> Result<Option<Secret<T>>> {
        match SecretsManager::get_secret_value(
            &self.secrets_manager_client,
            GetSecretValueRequest {
                secret_id: name.to_string(),
                version_id: version.map(String::from),
                version_stage: None,
            },
        )
        .await
        {
            Ok(response) => Self::parse_response(name, response).map(Some),
            Err(rusoto_core::RusotoError::Service(GetSecretValueError::ResourceNotFound(_))) => {
                Ok(None)
            }
            Err(e) => Err(SecretsProviderError::ProviderFailed(e.to_string())),
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

    // NOTE: Rusoto does not support batch get secret value method, so we'll just
    // leave the default implementation in place.
}
