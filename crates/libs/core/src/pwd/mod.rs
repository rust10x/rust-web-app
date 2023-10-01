// region:    --- Modules

mod error;
mod scheme;

pub use self::error::{Error, Result};

use crate::pwd::scheme::{get_scheme, SchemeStatus, DEFAULT_SCHEME};
use lazy_regex::regex_captures;

// endregion: --- Modules

// region:    --- Types

pub struct EncryptContent {
	pub content: String, // Clear content.
	pub salt: String,    // Clear salt.
}

// endregion: --- Types

// region:    --- Public Functions

/// Encrypt the password with the default scheme.
pub fn encrypt_pwd(enc_content: &EncryptContent) -> Result<String> {
	encrypt_for_scheme(DEFAULT_SCHEME, enc_content)
}

/// Validate if an EncryptContent matches.
pub fn validate_pwd(
	enc_content: &EncryptContent,
	pwd_with_scheme_ref: &str,
) -> Result<SchemeStatus> {
	let PwdParts {
		scheme_name,
		raw: raw_pwd_ref,
	} = parse_pwd(pwd_with_scheme_ref)?;

	validate_for_scheme(&scheme_name, enc_content, &raw_pwd_ref)?;

	if scheme_name == DEFAULT_SCHEME {
		Ok(SchemeStatus::Ok)
	} else {
		Ok(SchemeStatus::Outdated)
	}
}

// endregion: --- Public Functions

// region:    --- Scheme Infra

fn encrypt_for_scheme(
	scheme_name: &str,
	enc_content: &EncryptContent,
) -> Result<String> {
	let scheme = get_scheme(scheme_name)
		.map_err(|_| Error::SchemeUnknown(scheme_name.to_string()))?;

	let pwd_raw =
		scheme
			.encrypt(enc_content)
			.map_err(|scheme_error| Error::Scheme {
				scheme_name: scheme_name.to_string(),
				scheme_error,
			})?;

	Ok(format!("#{scheme_name}#{}", pwd_raw))
}

fn validate_for_scheme(
	scheme_name: &str,
	enc_content: &EncryptContent,
	raw_pwd_ref: &str,
) -> Result<()> {
	let scheme = get_scheme(scheme_name)
		.map_err(|_| Error::SchemeUnknown(scheme_name.to_string()))?;
	scheme
		.validate(enc_content, raw_pwd_ref)
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

fn parse_pwd(enc_content: &str) -> Result<PwdParts> {
	regex_captures!(
		r#"^#(\w+)#(.*)"#, // a literal regex
		enc_content
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

	#[test]
	fn test_validate() -> Result<()> {
		// -- Setup & Fixtures
		let fx_salt = "some-salt";
		let fx_pwd_clear = "welcome";

		let pwd_enc_1 = encrypt_pwd(&EncryptContent {
			salt: fx_salt.to_string(),
			content: fx_pwd_clear.to_string(),
		})?;

		validate_pwd(
			&EncryptContent {
				salt: fx_salt.to_string(),
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
}
// endregion: --- Tests
