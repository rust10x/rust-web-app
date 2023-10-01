// region:    --- Modules

mod error;
pub mod scheme_01;
pub mod scheme_02;

pub use self::error::{Error, Result};
use crate::pwd::EncryptContent;

// endregion: --- Modules

pub const DEFAULT_SCHEME: &str = "01";

pub fn get_scheme(scheme_name: &str) -> Result<Box<dyn Scheme>> {
	match scheme_name {
		"01" => Ok(Box::new(scheme_01::Scheme01)),
		"02" => Ok(Box::new(scheme_02::Scheme02)),
		_ => Err(Error::SchemeNotFound),
	}
}

pub trait Scheme {
	fn encrypt(&self, enc_content: &EncryptContent) -> Result<String>;

	fn validate(
		&self,
		enc_content: &EncryptContent,
		raw_pwd_ref: &str,
	) -> Result<()>;
}

/// SchemeStatus is the return value of validate_pwd telling the caller if the
/// password scheme needs to updated.
#[derive(Debug)]
pub enum SchemeStatus {
	Ok,       // The pwd use the latest scheme. All good.
	Outdated, // The pwd use a old scheme. Would need to be re-encrypted.
}
