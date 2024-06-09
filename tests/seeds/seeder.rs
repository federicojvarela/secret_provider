use crate::{helpers::test_ext::SecretsProviderTestExt, seeds::constants::*};

pub async fn seed_secrets_provider(provider: &mut impl SecretsProviderTestExt) {
    provider.add_string_secret(SECRET_1_NAME, SECRET_1).await;
    provider.add_string_secret(SECRET_2_NAME, SECRET_2).await;
    provider.add_string_secret(SECRET_3_NAME, SECRET_3).await;
    provider.add_binary_secret(SECRET_4_NAME, SECRET_4).await;
    provider.add_binary_secret(SECRET_5_NAME, SECRET_5).await;
    provider.add_binary_secret(SECRET_6_NAME, SECRET_6).await;
    provider
        .add_string_secret(VERSIONED_SECRET_NAME, VERSIONED_SECRET_VERSION_1)
        .await;
    provider
        .add_string_secret(VERSIONED_SECRET_NAME, VERSIONED_SECRET_VERSION_2)
        .await;
}
