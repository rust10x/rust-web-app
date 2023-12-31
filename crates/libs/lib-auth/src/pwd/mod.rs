//! The pwd module is responsible for hashing and validating hashes.
//! It follows a multi-scheme hashing code design, allowing each
//! scheme to provide its own hashing and validation methods.
//!
//! Code Design Points:
//!
//! - Exposes two public async functions `hash_pwd(...)` and `validate_pwd(...)`
//! - `ContentToHash` represents the data to be hashed along with the corresponding salt.
//! - `SchemeStatus` is the result of `validate_pwd` which, upon successful validation, indicates
//!   whether the password needs to be re-hashed to adopt the latest scheme.
//! - Internally, the `pwd` module implements a multi-scheme code design with the `Scheme` trait.
//! - The `Scheme` trait exposes sync functions `hash` and `validate` to be implemented for each scheme.
//! - The two public async functions `hash_pwd(...)` and `validate_pwd(...)` call the scheme using
//!   `spawn_blocking` to ensure that long hashing/validation processes do not hinder the execution of smaller tasks.
//! - Schemes are designed to be agnostic of whether they are in an async or sync context, hence they are async-free.

// region:    --- Modules;

mod error;
mod scheme;

pub use self::error::{Error, Result};
pub use scheme::SchemeStatus;

use crate::pwd::scheme::{get_scheme, Scheme, DEFAULT_SCHEME};
use lazy_regex::regex_captures;
use std::str::FromStr;
use uuid::Uuid;

// endregion: --- Modules

// region:    --- Types

/// The clean content to hash, with the salt.
///
/// Notes:
///    - Since content is sensitive information, we do NOT implement default debug for this struct.
///    - The clone is only implement for testing
#[cfg_attr(test, derive(Clone))]
pub struct ContentToHash {
	pub content: String, // Clear content.
	pub salt: Uuid,
}

// endregion: --- Types

// region:    --- Public Functions

/// Hash the password with the default scheme.
pub async fn hash_pwd(to_hash: ContentToHash) -> Result<String> {
	tokio::task::spawn_blocking(move || hash_for_scheme(DEFAULT_SCHEME, to_hash))
		.await
		.map_err(|_| Error::FailSpawnBlockForHash)?
}

/// Validate if an ContentToHash matches.
pub async fn validate_pwd(
	to_hash: ContentToHash,
	pwd_ref: String,
) -> Result<SchemeStatus> {
	let PwdParts {
		scheme_name,
		hashed,
	} = pwd_ref.parse()?;

	// Note: We do first, so that we do not have to clonse the scheme_name.
	let scheme_status = if scheme_name == DEFAULT_SCHEME {
		SchemeStatus::Ok
	} else {
		SchemeStatus::Outdated
	};

	// Note: Since validate might take some time depending on algo
	//       doing a spawn_blocking to avoid
	tokio::task::spawn_blocking(move || {
		validate_for_scheme(&scheme_name, to_hash, hashed)
	})
	.await
	.map_err(|_| Error::FailSpawnBlockForValidate)??;

	// validate_for_scheme(&scheme_name, to_hash, &hashed).await?;
	Ok(scheme_status)
}
// endregion: --- Public Functions

// region:    --- Privates

fn hash_for_scheme(scheme_name: &str, to_hash: ContentToHash) -> Result<String> {
	let pwd_hashed = get_scheme(scheme_name)?.hash(&to_hash)?;

	Ok(format!("#{scheme_name}#{pwd_hashed}"))
}

fn validate_for_scheme(
	scheme_name: &str,
	to_hash: ContentToHash,
	pwd_ref: String,
) -> Result<()> {
	get_scheme(scheme_name)?.validate(&to_hash, &pwd_ref)?;
	Ok(())
}

struct PwdParts {
	/// The scheme only (e.g., "01")
	scheme_name: String,
	/// The hashed password,
	hashed: String,
}

impl FromStr for PwdParts {
	type Err = Error;

	fn from_str(pwd_with_scheme: &str) -> Result<Self> {
		regex_captures!(
			r#"^#(\w+)#(.*)"#, // a literal regex
			pwd_with_scheme
		)
		.map(|(_, scheme, hashed)| Self {
			scheme_name: scheme.to_string(),
			hashed: hashed.to_string(),
		})
		.ok_or(Error::PwdWithSchemeFailedParse)
	}
}

// endregion: --- Privates

// region:    --- Tests
#[cfg(test)]
mod tests {
	pub type Result<T> = core::result::Result<T, Error>;
	pub type Error = Box<dyn std::error::Error>; // For tests.

	use super::*;

	#[tokio::test]
	async fn test_multi_scheme_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_salt = Uuid::parse_str("f05e8961-d6ad-4086-9e78-a6de065e5453")?;
		let fx_to_hash = ContentToHash {
			content: "hello world".to_string(),
			salt: fx_salt,
		};

		// -- Exec
		let pwd_hashed = hash_for_scheme("01", fx_to_hash.clone())?;
		let pwd_validate = validate_pwd(fx_to_hash.clone(), pwd_hashed).await?;

		// -- Check
		assert!(
			matches!(pwd_validate, SchemeStatus::Outdated),
			"status should be SchemeStatus::Outdated"
		);

		Ok(())
	}
}
// endregion: --- Tests
