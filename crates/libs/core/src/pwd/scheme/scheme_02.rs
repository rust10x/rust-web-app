use super::{Error, Result};
use crate::config;
use crate::pwd::scheme::Scheme;
use crate::pwd::ContentToHash;
use argon2::password_hash::SaltString;
use argon2::{
	Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier,
	Version,
};
use lib_base::uuid::uuid_to_b64;
use std::sync::OnceLock;

pub struct Scheme02;

impl Scheme for Scheme02 {
	fn hash(&self, to_hash: &ContentToHash) -> Result<String> {
		let argon2 = get_argon2();

		// TODO: Might want to have ContentToHash.salt as Uuid, to avoid the back and forth.
		let salt_b64 = SaltString::from_b64(&uuid_to_b64(to_hash.salt))
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
			Algorithm::Argon2id,
			Version::V0x13,
			Params::default(),
		)
		.unwrap() // FIXME: Needs to remove that, and probably return an Result
	});

	val
}
