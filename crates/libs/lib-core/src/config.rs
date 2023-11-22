use lib_utils::envs::get_env;
use std::sync::OnceLock;

pub fn core_config() -> &'static CoreConfig {
	static INSTANCE: OnceLock<CoreConfig> = OnceLock::new();

	INSTANCE.get_or_init(|| {
		CoreConfig::load_from_env().unwrap_or_else(|ex| {
			panic!("FATAL - WHILE LOADING CONF - Cause: {ex:?}")
		})
	})
}

#[allow(non_snake_case)]
pub struct CoreConfig {
	// -- Db
	pub DB_URL: String,

	// -- Web
	pub WEB_FOLDER: String,
}

impl CoreConfig {
	fn load_from_env() -> lib_utils::envs::Result<CoreConfig> {
		Ok(CoreConfig {
			// -- Db
			DB_URL: get_env("SERVICE_DB_URL")?,

			// -- Web
			WEB_FOLDER: get_env("SERVICE_WEB_FOLDER")?,
		})
	}
}
