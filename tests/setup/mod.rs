#[cfg(feature = "memory")]
pub mod memory;

#[cfg(feature = "legacy-rusoto-aws")]
pub mod rusoto;

#[cfg(feature = "aws")]
pub mod aws;
