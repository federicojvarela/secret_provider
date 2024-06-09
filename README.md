# Secrets Provider crate

The objetive of this crate is to abstract away secret managment solutions and have a common interface for all of them.

## Usage

To use this library in your project you must include the following line in your `Cargo.toml`:

```toml
secrets_provider = { git = "ssh://git@github.com/federicojvarela/secrets_provider/", tag = "<<LIBRARY_VERSION>>", features = ["aws"] }
```
where `<<LIBRARY_VERSION>>` is the git tag of the version you want to use.

Currently, the lib support two features:
- `aws`: Enables the Secret Provider implementation for AWS.
- `memory`: Enables the memory Secret Provider implementation.

## Testing

### Memory implementation

To test the memory implementation, you should run:
```bash
$ cargo test --features memory
```

### Amazon Web Services

To test the AWS implementation, first you must run the AWS Secret Manager emulators by running:
```bash
$ docker-compose up
```

After the emulators are running and seeded (check if the seeder container finished) you should run:
```bash
$ cargo test --features aws
```

## Documentation

To generate and open the Rust documentation you should run:
```bash
$ cargo doc --no-deps --features aws,memory --open
```

## Supported secret types

Right now two datatypes are supported for secrets:
- string: Represented with the `String` type.
- binary: Represented with the `Vec<u8>` type.

This means that you have to explicitly type the function `get_secret`  with turbofish (`::<T>`) or use it in a context where the type can be inferred.

## Implementations

### Amazon Web Services

In order to connect to the real AWS, the following

* `AWS_WEB_IDENTITY_TOKEN_FILE` - Path to the web identity token file.
* `AWS_ROLE_ARN` - ARN of the role to assume.
* `AWS_ROLE_SESSION_NAME` - **(optional)** name applied to the assume-role session.

[Click here for more information](https://docs.rs/rusoto_sts/0.45.0/rusoto_sts/struct.WebIdentityProvider.html#method.from_k8s_env)

### Example

```rust
use secrets_provider::SecretsProvider;
use secrets_provider::implementations::AwsSecretsProviderBuilder;

fn get_secrets_provider(region: String, endpoint: Option<String>) -> impl SecretsProvider {
    let mut builder = AwsSecretsProviderBuilder::new(region);

    if let Some(e) = endpoint {
        builder = builder.endpoint_override(e);
    }

    builder
        .build()
        .expect("Unable to initialize secrets provider")
}

#[tokio::main]
async fn main() {
    let secrets_provider = get_secrets_provider();
    let string_secret = secrets_provider
        .get::<String>("master_key_of_everything", None)
        .await
        .expect("There was an error getting the Master Key of Everything")
        .reveal();

    println!("The secret is: {}", string_secret);
}
```

### Memory

There is a Memory Secret Provider implementation. This **should never** be used to save real secrets. It is just for testing purposes.

### Example

```rust
use secrets_provider::SecretsProvider;
use secrets_provider::implementations::MemorySecretsProvider;

fn get_secrets_provider(region: String, endpoint: Option<String>) -> impl SecretsProvider {
    MemorySecretsProvider::new()
}

#[tokio::main]
async fn main() {
    let secrets_provider = get_secrets_provider();
    let string_secret = secrets_provider
        .get::<String>("master_key_of_everything", None)
        .await
        .expect("There was an error getting the Master Key of Everything")
        .reveal();

    println!("The secret is: {}", string_secret);
}
```

