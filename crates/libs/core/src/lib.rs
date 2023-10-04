mod config;
pub mod ctx;
pub mod model;
pub mod pwd;
pub mod token;

// #[cfg(test)] // Commented during early development.
pub mod _dev_utils;

pub use config::config;

// Generic Wrapper tuple struct for newtype pattern,
// mostly for external type to type From/TryFrom conversions
// Usage `use crate::W`
pub struct W<T>(pub T);
