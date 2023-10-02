use super::{Error, Result};
use crate::config;
use crate::pwd::scheme::Scheme;
use crate::pwd::ContentToHash;
use argon2::password_hash::SaltString;
use argon2::{
	Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier,
	Version,
};
use std::sync::OnceLock;

pub struct Scheme02;

impl Scheme for Scheme02 {
	fn hash(&self, to_hash: &ContentToHash) -> Result<String> {
		let argon2 = get_argon2();

		// NOTE: SaltString::from_b64(..) takes a string, but if "=" it returns error
		//       So, using the SaltString::encode_b64(&[u8])
		let salt_b64 = SaltString::encode_b64(to_hash.salt.as_bytes())
			.map_err(|_| Error::Salt)?;

		let pwd = argon2
			.hash_password(to_hash.content.as_bytes(), &salt_b64)
			.map_err(|_| Error::Hash)?
			.to_string();

		Ok(pwd)
	}

	fn validate(&self, to_hash: &ContentToHash, raw_pwd_ref: &str) -> Result<()> {
		let argon2 = get_argon2();

		// NOTE: For now, we validate using the password itself rather than re-hashing it.
		//       The downside is that if a different scheme based on Argon2 uses a different Argon2 config,
		//       it might still pass since the configs are parsed from the password.
		//       It might be worth considering whether it's more strict to simply re-hash the password
		//       and perform a straightforward comparison. This is because schemes should be as strict as possible,
		//       and we may want different schemes with varying Argon2 parameters.

		let parsed_hash_ref =
			PasswordHash::new(raw_pwd_ref).map_err(|_| Error::Hash)?;

		argon2
			.verify_password(to_hash.content.as_bytes(), &parsed_hash_ref)
			.map_err(|_| Error::PwdValidate)
	}
}

/// Note: The `argon2` crate allows to reuse the `Argon2` object for password hashing.
fn get_argon2() -> &'static Argon2<'static> {
	static INSTANCE: OnceLock<Argon2<'static>> = OnceLock::new();

	let val = INSTANCE.get_or_init(|| {
		let key = &config().PWD_KEY;

		Argon2::new_with_secret(
			key,
			Algorithm::Argon2id, // Same as Argon2::default()
			Version::V0x13,      // Same as Argon2::default()
			Params::default(),
		)
		.unwrap() // FIXME: Needs to remove that, and probably return an Result
	});

	val
}

// region:    --- Tests
#[cfg(test)]
mod tests {
	use super::*;
	use anyhow::Result;
	use lib_base::uuid::uuid_parse;

	#[test]
	fn test_scheme_02_hash_into_b64u_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_to_hash = ContentToHash {
			content: "hello world".to_string(),
			salt: uuid_parse("f05e8961-d6ad-4086-9e78-a6de065e5453")?,
		};
		let fx_res = "$argon2id$v=19$m=19456,t=2,p=1$8F6JYdatQIaeeKbeBl5UUw$TaRnmmbDdQ1aTzk2qQ2yQzPQoZfnKqhrfuTH/TRP5V4";

		// -- Exec
		let scheme = Scheme02;
		let res = scheme.hash(&fx_to_hash)?;

		// -- Check
		assert_eq!(res, fx_res);

		Ok(())
	}
}
// endregion: --- Tests
