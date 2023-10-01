use crate::pwd::scheme;
use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize)]
pub enum Error {
	// -- Key
	KeyFailHmac,

	Scheme {
		scheme_name: String,
		scheme_error: scheme::Error,
	},

	// -- Pwd
	NotMatching,
	SchemeUnknown(String),
	SchemeNotInContent,
}

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}
// endregion: --- Error Boilerplate