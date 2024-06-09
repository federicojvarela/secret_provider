/// Use AWS official SDK
#[cfg(feature = "aws")]
pub mod aws;

/// Use Rusoto SDK
#[cfg(feature = "legacy-rusoto-aws")]
pub mod rusoto;

/// Use a dummy in-memory secrets provider
#[cfg(feature = "memory")]
pub mod memory;
