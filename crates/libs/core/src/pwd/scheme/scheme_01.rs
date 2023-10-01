use super::{Error, Result};
use crate::config;
use crate::pwd::scheme::Scheme;
use crate::pwd::EncryptContent;
use hmac::{Hmac, Mac};
use lib_base::b64::b64u_encode;
use sha2::Sha512;

pub struct Scheme01;

impl Scheme for Scheme01 {
	fn encrypt(&self, enc_content: &EncryptContent) -> Result<String> {
		let key = &config().PWD_KEY;
		encrypt(key, enc_content)
	}

	fn validate(
		&self,
		enc_content: &EncryptContent,
		raw_pwd_ref: &str,
	) -> Result<()> {
		let raw_pwd_new = self.encrypt(enc_content)?;
		if raw_pwd_new == raw_pwd_ref {
			Ok(())
		} else {
			Err(Error::PwdValidate)
		}
	}
}

fn encrypt(key: &[u8], enc_content: &EncryptContent) -> Result<String> {
	let EncryptContent { content, salt } = enc_content;

	// -- Create a HMAC-SHA-512 from key.
	let mut hmac_sha512 =
		Hmac::<Sha512>::new_from_slice(key).map_err(|_| Error::Key)?;

	// -- Add content.
	hmac_sha512.update(content.as_bytes());
	hmac_sha512.update(salt.as_bytes());

	// -- Finalize and b64u encode.
	let hmac_result = hmac_sha512.finalize();
	let result_bytes = hmac_result.into_bytes();

	let result = b64u_encode(result_bytes);

	Ok(result)
}

// region:    --- Tests
#[cfg(test)]
mod tests {
	use super::*;
	use anyhow::Result;
	use rand::RngCore;

	#[test]
	fn test_encrypt_into_b64u_ok() -> Result<()> {
		// -- Setup & Fixture
		let mut fx_key = [0u8; 64]; // 512 bits = 64 bytes
		rand::thread_rng().fill_bytes(&mut fx_key);
		let fx_enc_content = EncryptContent {
			content: "hello world".to_string(),
			salt: "some pepper".to_string(),
		};
		// TODO: Need to fix fx_key, and precompute fx_res.
		let fx_res = encrypt(&fx_key, &fx_enc_content)?;

		// -- Exec
		let res = encrypt(&fx_key, &fx_enc_content)?;

		// -- Check
		assert_eq!(res, fx_res);

		Ok(())
	}
}
// endregion: --- Tests
