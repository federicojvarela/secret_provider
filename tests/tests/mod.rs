#[cfg(feature = "aws")]
mod aws;
#[cfg(feature = "memory")]
mod memory;
#[cfg(feature = "legacy-rusoto-aws")]
mod rusoto;

#[macro_use]
mod generic;
