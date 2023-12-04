// region:    --- Modules

mod error;
mod scheme_01;
mod scheme_02;

pub use self::error::{Error, Result};

use crate::pwd::ContentToHash;
use enum_dispatch::enum_dispatch;

// endregion: --- Modules

pub const DEFAULT_SCHEME: &str = "02";

#[derive(Debug)]
pub enum SchemeStatus {
	Ok,       // The pwd uses the latest scheme. All good.
	Outdated, // The pwd uses an old scheme.
}

#[enum_dispatch]
pub trait Scheme {
	fn hash(&self, to_hash: &ContentToHash) -> Result<String>;

	fn validate(&self, to_hash: &ContentToHash, pwd_ref: &str) -> Result<()>;
}

#[enum_dispatch(Scheme)]
pub enum SchemeDispatcher {
	Scheme01(scheme_01::Scheme01),
	Scheme02(scheme_02::Scheme02),
}

pub fn get_scheme(scheme_name: &str) -> Result<impl Scheme> {
	match scheme_name {
		"01" => Ok(SchemeDispatcher::Scheme01(scheme_01::Scheme01)),
		"02" => Ok(SchemeDispatcher::Scheme02(scheme_02::Scheme02)),
		_ => Err(Error::SchemeNotFound(scheme_name.to_string())),
	}
}
