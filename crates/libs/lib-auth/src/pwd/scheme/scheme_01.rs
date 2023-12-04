use super::{Error, Result};
use crate::auth_config;
use crate::pwd::scheme::Scheme;
use crate::pwd::ContentToHash;
use hmac::{Hmac, Mac};
use lib_utils::b64::b64u_encode;
use sha2::Sha512;

pub struct Scheme01;

impl Scheme for Scheme01 {
	fn hash(&self, to_hash: &ContentToHash) -> Result<String> {
		let key = &auth_config().PWD_KEY;
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
	use crate::auth_config;
	use anyhow::Result;
	use uuid::Uuid;

	#[test]
	fn test_scheme_01_hash_into_b64u_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_salt = Uuid::parse_str("f05e8961-d6ad-4086-9e78-a6de065e5453")?;
		let fx_key = &auth_config().PWD_KEY; // 512 bits = 64 bytes
		let fx_to_hash = ContentToHash {
			content: "hello world".to_string(),
			salt: fx_salt,
		};
		// TODO: Need to fix fx_key, and precompute fx_res.
		let fx_res = "qO9A90161DoewhNXFwVcnAaljRIVnajvd5zsVDrySCwxpoLwVCACzaz-8Ev2ZpI8RackUTLBVqFI6H5oMe-OIg";

		// -- Exec
		let res = hash(fx_key, &fx_to_hash)?;

		// -- Check
		assert_eq!(res, fx_res);

		Ok(())
	}
}
// endregion: --- Tests
