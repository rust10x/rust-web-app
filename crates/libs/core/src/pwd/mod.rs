// region:    --- Modules

mod error;
mod scheme;

pub use self::error::{Error, Result};

use crate::pwd::scheme::{get_scheme, SchemeStatus, DEFAULT_SCHEME};
use lazy_regex::regex_captures;
use uuid::Uuid;

// endregion: --- Modules

// region:    --- Types

pub struct ContentToHash {
	pub content: String, // Clear content.
	pub salt: Uuid,      // Clear salt.
}

// endregion: --- Types

// region:    --- Public Functions

/// hash the password with the default scheme.
pub fn hash_pwd(to_hash: &ContentToHash) -> Result<String> {
	hash_for_scheme(DEFAULT_SCHEME, to_hash)
}

/// Validate if an ContentToHash matches.
pub fn validate_pwd(
	to_hash: &ContentToHash,
	pwd_with_scheme_ref: &str,
) -> Result<SchemeStatus> {
	let PwdParts {
		scheme_name,
		raw: raw_pwd_ref,
	} = parse_pwd(pwd_with_scheme_ref)?;

	validate_for_scheme(&scheme_name, to_hash, &raw_pwd_ref)?;

	if scheme_name == DEFAULT_SCHEME {
		Ok(SchemeStatus::Ok)
	} else {
		Ok(SchemeStatus::Outdated)
	}
}

// endregion: --- Public Functions

// region:    --- Scheme Infra

fn hash_for_scheme(scheme_name: &str, to_hash: &ContentToHash) -> Result<String> {
	let scheme = get_scheme(scheme_name)
		.map_err(|_| Error::SchemeUnknown(scheme_name.to_string()))?;

	let pwd_raw = scheme.hash(to_hash).map_err(|scheme_error| Error::Scheme {
		scheme_name: scheme_name.to_string(),
		scheme_error,
	})?;

	Ok(format!("#{scheme_name}#{}", pwd_raw))
}

fn validate_for_scheme(
	scheme_name: &str,
	to_hash: &ContentToHash,
	raw_pwd_ref: &str,
) -> Result<()> {
	let scheme = get_scheme(scheme_name)
		.map_err(|_| Error::SchemeUnknown(scheme_name.to_string()))?;
	scheme
		.validate(to_hash, raw_pwd_ref)
		.map_err(|scheme_error| Error::Scheme {
			scheme_name: scheme_name.to_string(),
			scheme_error,
		})
}

struct PwdParts {
	/// The scheme only (e.g. "01")
	scheme_name: String,
	/// The raw password, without the scheme name.
	raw: String,
}

fn parse_pwd(pwd_with_scheme: &str) -> Result<PwdParts> {
	regex_captures!(
		r#"^#(\w+)#(.*)"#, // a literal regex
		pwd_with_scheme
	)
	.map(|(_whole, scheme, raw)| PwdParts {
		scheme_name: scheme.to_string(),
		raw: raw.to_string(),
	})
	.ok_or(Error::SchemeNotInContent)
}

// endregion: --- Scheme Infra

// region:    --- Tests
#[cfg(test)]
mod tests {
	use super::*;
	use anyhow::Result;
	use lib_base::uuid::uuid_parse;

	#[test]
	fn test_validate() -> Result<()> {
		// -- Setup & Fixtures
		let fx_salt = uuid_parse("f05e8961-d6ad-4086-9e78-a6de065e5453")?;
		let fx_pwd_clear = "welcome";

		let pwd_enc_1 = hash_pwd(&ContentToHash {
			salt: fx_salt,
			content: fx_pwd_clear.to_string(),
		})?;

		validate_pwd(
			&ContentToHash {
				salt: fx_salt,
				content: fx_pwd_clear.to_string(),
			},
			&pwd_enc_1,
		)?;

		Ok(())
	}

	#[test]
	fn test_parse_pwd_ok() -> Result<()> {
		// -- Fixtures
		let fx_pwd = "#01#DdVzPPKKpjs-xuf-Y88t3MpQ5KPDqa7C2gpaTIysHnHIzX_j2IgNb3WtEDHLfF2ps1OWVPKOkgLFvvDMvNrN-A";

		// -- Exec
		let PwdParts { scheme_name, .. } = parse_pwd(fx_pwd)?;

		// -- Check
		assert_eq!(scheme_name, "01");

		Ok(())
	}

	#[test]
	fn test_parse_pwd_err_without() -> Result<()> {
		// -- Fixtures
		let fx_pwd = "DdVzPPKKpjs-xuf-Y88t3MpQ5KPDqa7C2gpaTIysHnHIzX_j2IgNb3WtEDHLfF2ps1OWVPKOkgLFvvDMvNrN-A";

		// -- Exec
		let res = parse_pwd(fx_pwd);

		// -- Check
		assert!(
			matches!(res, Err(Error::SchemeNotInContent)),
			"Error not matching. Should have been SchemeNotFoundInContent"
		);

		Ok(())
	}

	#[test]
	fn test_scheme_upgrade_ok() -> Result<()> {
		// -- Fixtures
		let fx_salt = uuid_parse("f05e8961-d6ad-4086-9e78-a6de065e5453")?;
		let fx_pwd_clear = "welcome";
		let fx_to_hash = ContentToHash {
			content: fx_pwd_clear.to_string(),
			salt: fx_salt,
		};
		let fx_02_pwd = "#02#$argon2id$v=19$m=19456,t=2,p=1$8F6JYdatQIaeeKbeBl5UUw$H0HJXVHWXDh/B1BOlY+ov5hBNc8Sd434KERqgljWSxk";

		// -- scheme 01 hash & validate & check
		let pwd_01 = hash_for_scheme("01", &fx_to_hash)?;
		let scheme_status = validate_pwd(&fx_to_hash, &pwd_01)?;
		assert!(
			matches!(scheme_status, SchemeStatus::Outdated),
			"scheme 01 should be Outdated"
		);

		// -- normal hash & validate & check
		let pwd_default = hash_pwd(&fx_to_hash)?;
		assert_eq!(pwd_default, fx_02_pwd);
		let scheme_status = validate_pwd(&fx_to_hash, &pwd_default)?;
		assert!(
			matches!(scheme_status, SchemeStatus::Ok),
			"scheme 02 should be Ok"
		);

		Ok(())
	}
}
// endregion: --- Tests
