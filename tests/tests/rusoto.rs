//! Tests that are specific to the AWS implementation using Rusoto.
//!
//! Unless your tests needs to interact directly with the AWS client, you should
//! create a generic test instead.

use rusoto_secretsmanager::{ListSecretVersionIdsRequest, SecretsManager as _};
use secrets_provider::SecretsProvider;

use crate::{
    generate_generic_tests,
    seeds::{constants::*, seeder::seed_secrets_provider},
};

// This line will include all generic tests using Rusoto implementation.
generate_generic_tests!(crate::setup::rusoto::load_test_provider().await);

#[tokio::test]
async fn test_can_retrieve_previous_and_current_aws_stages() {
    let mut secrets_provider = crate::setup::rusoto::load_test_provider().await;
    seed_secrets_provider(&mut secrets_provider).await;

    // Read the secrets versions directly from the manager. We have to do this because every time
    // the emulator is seeded, the versions change. So, we read the version_id for each version of
    // the versioned secret and use them to retrieve the secrets and tests that the versioned get
    // works
    let secret_versions = secrets_provider
        .client
        .list_secret_version_ids(ListSecretVersionIdsRequest {
            include_deprecated: None,
            max_results: None,
            next_token: None,
            secret_id: VERSIONED_SECRET_NAME.to_string(),
        })
        .await
        .unwrap();

    let mut versions = secret_versions.clone().versions.unwrap().into_iter();
    let previous_version = versions
        .find(|secret_version| {
            secret_version
                .version_stages
                .as_ref()
                .unwrap()
                .contains(&String::from("AWSPREVIOUS"))
        })
        .unwrap()
        .version_id
        .unwrap();

    let previous_secret = secrets_provider
        .find_with_version::<String>(VERSIONED_SECRET_NAME, &previous_version)
        .await
        .unwrap()
        .expect("Secret / version pair not found")
        .reveal();

    assert_eq!(previous_secret, VERSIONED_SECRET_VERSION_1);

    let mut versions = secret_versions.versions.unwrap().into_iter();
    let current_version = versions
        .find(|secret_version| {
            secret_version
                .version_stages
                .as_ref()
                .unwrap()
                .contains(&String::from("AWSCURRENT"))
        })
        .unwrap()
        .version_id
        .unwrap();

    let current_secret = secrets_provider
        .find_with_version::<String>(VERSIONED_SECRET_NAME, &current_version)
        .await
        .unwrap()
        .expect("Secret / version pair not found")
        .reveal();

    assert_eq!(current_secret, VERSIONED_SECRET_VERSION_2);
}
