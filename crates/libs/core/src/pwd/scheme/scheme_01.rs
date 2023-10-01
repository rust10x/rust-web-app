use super::{Error, Result};
use crate::config;
use crate::pwd::scheme::Scheme;
use crate::pwd::ContentToHash;
use hmac::{Hmac, Mac};
use lib_base::b64::b64u_encode;
use sha2::Sha512;

pub struct Scheme01;

impl Scheme for Scheme01 {
	fn hash(&self, to_hash: &ContentToHash) -> Result<String> {
		let key = &config().PWD_KEY;
		hash(key, to_hash)
	}

	fn validate(&self, to_hash: &ContentToHash, raw_pwd_ref: &str) -> Result<()> {
		let raw_pwd_new = self.hash(to_hash)?;
		if raw_pwd_new == raw_pwd_ref {
			Ok(())
		} else {
			Err(Error::PwdValidate)
		}
	}
}

fn hash(key: &[u8], to_hash: &ContentToHash) -> Result<String> {
	let ContentToHash { content, salt } = to_hash;

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
	use lib_base::uuid::uuid_parse;
	use rand::RngCore;

	#[test]
	fn test_hash_into_b64u_ok() -> Result<()> {
		// -- Setup & Fixture
		let mut fx_key = [0u8; 64]; // 512 bits = 64 bytes
		rand::thread_rng().fill_bytes(&mut fx_key);
		let fx_to_hash = ContentToHash {
			content: "hello world".to_string(),
			salt: uuid_parse("f05e8961-d6ad-4086-9e78-a6de065e5453")?,
		};
		// TODO: Need to fix fx_key, and precompute fx_res.
		let fx_res = hash(&fx_key, &fx_to_hash)?;

		// -- Exec
		let res = hash(&fx_key, &fx_to_hash)?;

		// -- Check
		assert_eq!(res, fx_res);

		Ok(())
	}
}
// endregion: --- Tests
