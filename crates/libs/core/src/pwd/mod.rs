// region:    --- Modules
mod error;

pub use self::error::{Error, Result};

use crate::config;
use hmac::{Hmac, Mac};
use lib_base::b64::b64u_encode;
use sha2::Sha512;
// endregion: --- Modules

pub struct EncryptContent {
	pub content: String, // Clear content.
	pub salt: String,    // Clear salt.
}
/// Encrypt the password with the default scheme.
pub fn encrypt_pwd(enc_content: &EncryptContent) -> Result<String> {
	let key = &config().PWD_KEY;

	let encrypted = encrypt_into_b64u(key, enc_content)?;

	Ok(format!("#01#{encrypted}"))
}

/// Validate if an EncryptContent matches.
pub fn validate_pwd(enc_content: &EncryptContent, pwd_ref: &str) -> Result<()> {
	let pwd = encrypt_pwd(enc_content)?;

	if pwd == pwd_ref {
		Ok(())
	} else {
		Err(Error::NotMatching)
	}
}

pub fn encrypt_into_b64u(
	key: &[u8],
	enc_content: &EncryptContent,
) -> Result<String> {
	let EncryptContent { content, salt } = enc_content;

	// -- Create a HMAC-SHA-512 from key.
	let mut hmac_sha512 =
		Hmac::<Sha512>::new_from_slice(key).map_err(|_| Error::KeyFailHmac)?;

	// -- Add content.
	hmac_sha512.update(content.as_bytes());
	hmac_sha512.update(salt.as_bytes());

	// -- Finalize and b64u encode.
	let hmac_result = hmac_sha512.finalize();

	let result = b64u_encode(hmac_result.into_bytes());

	Ok(result)
}
