// region:    --- Modules

mod error;
mod hmac_hasher;

pub use self::error::{Error, Result};

use crate::auth_config;
use crate::pwd::hmac_hasher::hmac_sha512_hash;
use uuid::Uuid;

// endregion: --- Modules

// region:    --- Types

pub struct ContentToHash {
	pub content: String, // Clear content.
	pub salt: Uuid,      // Clear salt.
}

// endregion: --- Types

// region:    --- Public Functions

/// Hash the password with the default scheme.
pub fn hash_pwd(to_hash: &ContentToHash) -> Result<String> {
	let key = &auth_config().PWD_KEY;

	let hashed = hmac_sha512_hash(key, to_hash)?;

	Ok(format!("#01#{hashed}"))
}

/// Validate if an ContentToHash matches.
pub fn validate_pwd(enc_content: &ContentToHash, pwd_ref: &str) -> Result<()> {
	let pwd = hash_pwd(enc_content)?;

	if pwd == pwd_ref {
		Ok(())
	} else {
		Err(Error::NotMatching)
	}
}

// endregion: --- Public Functions
