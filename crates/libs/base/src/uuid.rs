use base64::engine::{general_purpose, Engine};
use uuid::Uuid;

pub fn uuid_new() -> Uuid {
	Uuid::new_v4()
}

pub fn uuid_parse(val: &str) -> Result<Uuid> {
	Uuid::parse_str(val).map_err(|_| Error::FailParse)
}

pub fn uuid_to_b64(uuid: Uuid) -> String {
	general_purpose::STANDARD.encode(uuid.as_bytes())
}

// region:    --- Error
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
	FailParse,
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

// endregion: --- Error
