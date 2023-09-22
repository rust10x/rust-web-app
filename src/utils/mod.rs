// region:    --- Modules

mod error;

pub use self::error::{Error, Result};

use base64::engine::{general_purpose, Engine};
use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};

// endregion: --- Modules

// region:    --- Time
pub fn now_utc() -> OffsetDateTime {
	OffsetDateTime::now_utc()
}

pub fn format_time(time: OffsetDateTime) -> String {
	time.format(&Rfc3339).unwrap() // TODO: need to check if safe.
}

pub fn now_utc_plus_sec_str(sec: f64) -> String {
	let new_time = now_utc() + Duration::seconds_f64(sec);
	format_time(new_time)
}

pub fn parse_utc(moment: &str) -> Result<OffsetDateTime> {
	OffsetDateTime::parse(moment, &Rfc3339)
		.map_err(|_| Error::DateFailParse(moment.to_string()))
}
// endregion: --- Time

// region:    --- Base64
pub fn b64u_encode(content: impl AsRef<[u8]>) -> String {
	general_purpose::URL_SAFE_NO_PAD.encode(content)
}

pub fn b64u_decode(b64u: &str) -> Result<Vec<u8>> {
	general_purpose::URL_SAFE_NO_PAD
		.decode(b64u)
		.map_err(|_| Error::FailToB64uDecode)
}

pub fn b64u_decode_to_string(b64u: &str) -> Result<String> {
	b64u_decode(b64u)
		.ok()
		.and_then(|r| String::from_utf8(r).ok())
		.ok_or(Error::FailToB64uDecode)
}
// endregion: --- Base64
