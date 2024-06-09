//! Memory implementation of a Secrets Provider.
//!
//! Use this for testing purposes only!
use std::collections::HashMap;

use async_trait::async_trait;
use indexmap::IndexMap;
use uuid::Uuid;

use crate::{
    secret::{Decode, Secret, SecretData},
    Result, SecretsProvider,
};

enum MemorySecretType {
    Str(String),
    Bytes(Vec<u8>),
}

type Version = String;

pub struct MemorySecretsProvider {
    secrets: HashMap<String, IndexMap<String, MemorySecretType>>,
}

impl MemorySecretsProvider {
    pub fn new() -> Self {
        Self {
            secrets: HashMap::new(),
        }
    }

    pub fn add_binary_secret(&mut self, name: String, secret: Vec<u8>) -> Secret<Vec<u8>> {
        let version = Uuid::new_v4().to_string();
        if let Some(saved_secret) = self.secrets.get_mut(&name) {
            saved_secret.insert(
                Uuid::new_v4().to_string(),
                MemorySecretType::Bytes(secret.clone()),
            );
        } else {
            self.secrets.insert(
                name.clone(),
                IndexMap::from([(version.clone(), MemorySecretType::Bytes(secret.clone()))]),
            );
        }

        Secret {
            name,
            version,
            secret,
        }
    }

    pub fn add_string_secret(&mut self, name: String, secret: String) -> Secret<String> {
        let version = Uuid::new_v4().to_string();
        if let Some(saved_secret) = self.secrets.get_mut(&name) {
            saved_secret.insert(
                Uuid::new_v4().to_string(),
                MemorySecretType::Str(secret.clone()),
            );
        } else {
            self.secrets.insert(
                name.clone(),
                IndexMap::from([(version.clone(), MemorySecretType::Str(secret.clone()))]),
            );
        }

        Secret {
            name,
            version,
            secret,
        }
    }

    pub fn list_secret_version_ids(&self, secret_name: &str) -> Option<Vec<Version>> {
        if let Some(saved_secret) = self.secrets.get(secret_name) {
            // Return the most recent version last
            return Some(saved_secret.keys().cloned().collect());
        }
        None
    }

    fn get_secret_from_memory<T: Decode>(
        &self,
        name: &str,
        version: Option<String>,
    ) -> Result<Option<Secret<T>>> {
        if let Some((secret, version)) = self.secrets.get(name).and_then(|saved_secret| {
            let secret = version
                .map(|v| saved_secret.get_key_value(&v))
                .unwrap_or_else(|| saved_secret.last());
            secret.map(|(version, secret)| match secret {
                MemorySecretType::Bytes(s) => (SecretData::Bytes(s.to_vec()), version.to_owned()),
                MemorySecretType::Str(s) => (SecretData::Str(s.to_string()), version.to_owned()),
            })
        }) {
            Ok(Some(Secret {
                secret: T::decode(name, secret)?,
                name: name.to_string(),
                version,
            }))
        } else {
            Ok(None)
        }
    }
}

impl Default for MemorySecretsProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SecretsProvider for MemorySecretsProvider {
    async fn find<T: Decode>(&self, key_name: &str) -> Result<Option<Secret<T>>> {
        self.get_secret_from_memory(key_name, None)
    }

    async fn find_with_version<T: Decode>(
        &self,
        key_name: &str,
        version: &str,
    ) -> Result<Option<Secret<T>>> {
        self.get_secret_from_memory(key_name, Some(version.into()))
    }
}
