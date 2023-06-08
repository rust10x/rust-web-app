pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	// -- Time
	DateFailParse(String),

	// -- Base64
	FailToB64uDecode,
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
