// region:    --- Modules

mod config;
mod error;
mod web;

pub use self::error::{Error, Result};
use config::web_config;

use lib_web::middleware::mw_auth::{mw_ctx_require, mw_ctx_resolver};
use lib_web::middleware::mw_req_stamp::mw_req_stamp_resolver;
use lib_web::middleware::mw_res_map::mw_reponse_map;
use lib_web::routes::routes_static;

use crate::web::routes_login;

use axum::{middleware, Router};
use lib_core::_dev_utils;
use lib_core::model::ModelManager;
use tokio::net::TcpListener;
use tower_cookies::CookieManagerLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

// endregion: --- Modules

#[tokio::main]
async fn main() -> Result<()> {
	tracing_subscriber::fmt()
		.without_time() // For early local development.
		.with_target(false)
		.with_env_filter(EnvFilter::from_default_env())
		.init();

	// -- FOR DEV ONLY
	_dev_utils::init_dev().await;

	let mm = ModelManager::new().await?;

	// -- Define Routes
	let routes_rpc = web::routes_rpc::routes(mm.clone())
		.route_layer(middleware::from_fn(mw_ctx_require));

	let routes_all = Router::new()
		.merge(routes_login::routes(mm.clone()))
		.nest("/api", routes_rpc)
		.layer(middleware::map_response(mw_reponse_map))
		.layer(middleware::from_fn_with_state(mm.clone(), mw_ctx_resolver))
		.layer(CookieManagerLayer::new())
		.layer(middleware::from_fn(mw_req_stamp_resolver))
		.fallback_service(routes_static::serve_dir(&web_config().WEB_FOLDER));

	// region:    --- Start Server
	// Note: For this block, ok to unwrap.
	let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
	info!("{:<12} - {:?}\n", "LISTENING", listener.local_addr());
	axum::serve(listener, routes_all.into_make_service())
		.await
		.unwrap();
	// endregion: --- Start Server

	Ok(())
}
