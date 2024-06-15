//! This module encompasses errors for all `lib_rpc` modules and rpc handlers.
//! Variants from our application's library errors can be added as required by the handlers.
//!
//! # Note on the `rpc-router::Error` scheme
//!
//! - When used in an rpc handler with the `rpc-router` crate,
//!   this type will be encapsulated as a `rpc-router::Error::Handler(RpcHandlerError)` within a
//!   "TypeMap" and can subsequently be retrieved (see `web-server::web::Error` for reference).
//!
//! - For this application error to be utilized in the `rpc-router`, it must
//!   implement the `IntoRpcHandlerError` trait. This trait has a suitable default implementation,
//!   so simply adding `impl rpc_router::IntoRpcHandlerError for Error {}` would suffice.
//!
//! - Alternatively, the `#[derive(RpcHandlerError)]` annotation can be used as demonstrated here, which will
//!   automatically provide the `impl rpc_router::IntoRpcHandlerError for Error {}` for this type.

use derive_more::From;
use rpc_router::RpcHandlerError;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, From, Serialize, RpcHandlerError)]
pub enum Error {
	// -- App Libs
	#[from]
	Model(lib_core::model::Error),

	// -- External Modules
	#[from]
	SerdeJson(#[serde_as(as = "DisplayFromStr")] serde_json::Error),
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
