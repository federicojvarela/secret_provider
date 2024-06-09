/// Generate tests for a given implementation.
///
/// Usage:
///     generate_generic_tests(tests_module_name, get_mock_secrets_provider())
///
/// It will generate a new module called `tests_module_name` and call
/// `get_mock_secrets_provider()` before each test to get an instance
/// of the secrets provider.
///
/// Any expression resulting in an `impl SecretsProvider + SecretsProviderTestExt`
/// can be used in the place of `get_mock_secrets_provider()`. The expression can
/// be `async`.
#[macro_export]
macro_rules! generate_generic_tests {
    ($setup_fn:expr) => {
        mod generic {
            use secrets_provider::{SecretsProvider, SecretsProviderError};
            use $crate::{
                helpers::test_ext::SecretsProviderTestExt,
                seeds::{constants::*, seeder::seed_secrets_provider},
            };

            async fn get_secrets_provider() -> impl SecretsProvider + SecretsProviderTestExt {
                let mut provider = $setup_fn;
                seed_secrets_provider(&mut provider).await;
                provider
            }

            #[tokio::test]
            async fn can_read_string_secrets_correctly() {
                let secrets_provider = get_secrets_provider().await;

                let secret_1 = secrets_provider
                    .find::<String>(SECRET_1_NAME)
                    .await
                    .unwrap()
                    .expect("Secret not found");

                assert_eq!(SECRET_1_NAME, secret_1.name);
                assert_eq!(SECRET_1, secret_1.reveal());

                let secret_2 = secrets_provider
                    .find::<String>(SECRET_2_NAME)
                    .await
                    .unwrap()
                    .expect("Secret not found");

                assert_eq!(SECRET_2_NAME, secret_2.name);
                assert_eq!(SECRET_2, secret_2.reveal());

                let secret_3 = secrets_provider
                    .find::<String>(SECRET_3_NAME)
                    .await
                    .unwrap()
                    .expect("Secret not found");

                assert_eq!(SECRET_3_NAME, secret_3.name);
                assert_eq!(SECRET_3, secret_3.reveal());
            }

            #[tokio::test]
            async fn can_read_binary_secrets_correctly() {
                let secrets_provider = get_secrets_provider().await;

                let secret_4 = secrets_provider
                    .find::<Vec<u8>>(SECRET_4_NAME)
                    .await
                    .unwrap()
                    .expect("Secret not found");

                assert_eq!(SECRET_4_NAME, secret_4.name);
                assert_eq!(SECRET_4.to_vec(), secret_4.reveal());

                let secret_5 = secrets_provider
                    .find::<Vec<u8>>(SECRET_5_NAME)
                    .await
                    .unwrap()
                    .expect("Secret not found");

                assert_eq!(SECRET_5_NAME, secret_5.name);
                assert_eq!(SECRET_5.to_vec(), secret_5.reveal());

                let secret_6 = secrets_provider
                    .find::<Vec<u8>>(SECRET_6_NAME)
                    .await
                    .unwrap()
                    .expect("Secret not found");

                assert_eq!(SECRET_6_NAME, secret_6.name);
                assert_eq!(SECRET_6.to_vec(), secret_6.reveal());
            }

            #[tokio::test]
            async fn reading_binary_as_string_should_fail() {
                let secrets_provider = get_secrets_provider().await;

                let secret_4 = secrets_provider.find::<String>(SECRET_4_NAME).await;

                match secret_4 {
                    Err(SecretsProviderError::InvalidType(_)) => (),
                    r => panic!("Should have failed with InvalidType error: {:?}", r),
                }
            }

            #[tokio::test]
            async fn reading_string_as_binary_should_fail() {
                let secrets_provider = get_secrets_provider().await;

                let secret_1 = secrets_provider.find::<Vec<u8>>(SECRET_1_NAME).await;

                match secret_1 {
                    Err(SecretsProviderError::InvalidType(_)) => (),
                    r => panic!("Should have failed with InvalidType error: {:?}", r),
                }
            }

            #[tokio::test]
            async fn non_existent_secret_should_fail() {
                let secrets_provider = get_secrets_provider().await;

                let non_existent_secret = secrets_provider
                    .find::<Vec<u8>>("non-existent-secret")
                    .await
                    .unwrap();

                assert!(non_existent_secret.is_none())
            }

            #[tokio::test]
            async fn get_versioned_secrets_correctly() {
                let secrets_provider = get_secrets_provider().await;
                let secret_versions = secrets_provider
                    .list_secret_versions(VERSIONED_SECRET_NAME)
                    .await;

                let previous_secret = secrets_provider
                    .find_with_version::<String>(VERSIONED_SECRET_NAME, &secret_versions[0])
                    .await
                    .unwrap()
                    .expect("Secret / version pair not found")
                    .reveal();

                assert_eq!(previous_secret, VERSIONED_SECRET_VERSION_1);

                let current_secret = secrets_provider
                    .find_with_version::<String>(VERSIONED_SECRET_NAME, &secret_versions[1])
                    .await
                    .unwrap()
                    .expect("Secret / version pair not found")
                    .reveal();

                assert_eq!(current_secret, VERSIONED_SECRET_VERSION_2);
            }

            #[tokio::test]
            async fn find_inexistent_secret() {
                let secrets_provider = get_secrets_provider().await;

                let result_1 = secrets_provider
                    .find::<String>(SECRET_1_NAME)
                    .await
                    .unwrap();

                assert!(result_1.is_some());

                let result_2 = secrets_provider
                    .find::<String>("secret-that-does-NOT-exist")
                    .await
                    .unwrap();

                assert!(result_2.is_none());
            }

            #[tokio::test]
            async fn batch_find_works_all_existing() {
                let secrets_provider = get_secrets_provider().await;
                let mut retrieved = secrets_provider
                    .batch_find::<String>(&[SECRET_1_NAME, SECRET_2_NAME, SECRET_3_NAME])
                    .await
                    .unwrap();

                assert_eq!(retrieved.remove(SECRET_1_NAME).unwrap().reveal(), SECRET_1);
                assert_eq!(retrieved.remove(SECRET_2_NAME).unwrap().reveal(), SECRET_2);
                assert_eq!(retrieved.remove(SECRET_3_NAME).unwrap().reveal(), SECRET_3);
                assert!(retrieved.is_empty());
            }

            #[tokio::test]
            async fn batch_find_works_some_missing() {
                let secrets_provider = get_secrets_provider().await;
                let mut retrieved = secrets_provider
                    .batch_find::<String>(&[SECRET_1_NAME, "missing", SECRET_3_NAME])
                    .await
                    .unwrap();

                assert_eq!(retrieved.remove(SECRET_1_NAME).unwrap().reveal(), SECRET_1);
                assert!(retrieved.remove("missing").is_none());
                assert_eq!(retrieved.remove(SECRET_3_NAME).unwrap().reveal(), SECRET_3);
                assert!(retrieved.is_empty());
            }

            #[tokio::test]
            async fn batch_find_fails_for_mixed_types() {
                let secrets_provider = get_secrets_provider().await;
                let retrieved = secrets_provider
                    // Secret 4 is binary, Secret 1 is string
                    .batch_find::<Vec<u8>>(&[SECRET_1_NAME, SECRET_4_NAME])
                    .await;

                assert!(
                    retrieved.is_err(),
                    "Should've failed, instead returned: {retrieved:?}"
                );

                let retrieved = secrets_provider
                    // Secret 4 is binary, Secret 1 is string
                    .batch_find::<String>(&[SECRET_1_NAME, SECRET_4_NAME])
                    .await;

                assert!(
                    retrieved.is_err(),
                    "Should've failed, instead returned: {retrieved:?}"
                );
            }
        }
    };
}
