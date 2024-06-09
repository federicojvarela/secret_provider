//! Tests that are specific to the Memory implementation.
//!
//! It's very unlikely that you'd need to create a specific test for this implementation.
//! Create a generic test instead.

use crate::generate_generic_tests;

// Include all generic tests using Memory implementation.
generate_generic_tests!(crate::setup::memory::load_test_provider());
