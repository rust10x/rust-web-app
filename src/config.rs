use clap::Parser;
use std::sync::OnceLock;

pub fn config() -> &'static Config {
	static INSTANCE: OnceLock<Config> = OnceLock::new();

	INSTANCE.get_or_init(|| Config::parse())
}

/// Type alias for a byte vector.
///
/// This is needed because if you specify `Vec<u8>`, then clap will assume you want multiple `u8`
/// arguments. By aliasing this, we tell clap that instead we want a single argument that is a byte
/// vector.
type ByteVec = Vec<u8>;

/// Config options.
#[derive(Parser, Debug)]
pub struct Config {
	/// Password key.
	#[clap(long, env = "SERVICE_PWD_KEY", value_parser = base64_url::decode)]
	pub pwd_key: ByteVec,

	/// Token key.
	#[clap(long, env = "SERVICE_TOKEN_KEY", value_parser = base64_url::decode)]
	pub token_key: ByteVec,

	/// Token lifetime duration.
	#[clap(long, env = "SERVICE_TOKEN_DURATION_SEC")]
	pub token_duration_sec: f64,

	/// Database URL.
	#[clap(long, env = "SERVICE_DB_URL")]
	pub db_url: String,

	/// Folder for web assets.
	#[clap(long, env = "SERVICE_WEB_FOLDER")]
	pub web_folder: String,
}
