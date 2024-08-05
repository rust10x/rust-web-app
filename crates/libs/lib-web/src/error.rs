use crate::middleware;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use derive_more::From;
use lib_auth::{pwd, token};
use lib_core::model;
use serde::Serialize;
use serde_json::Value;
use serde_with::{serde_as, DisplayFromStr};
use std::sync::Arc;
use tracing::{debug, warn};

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize, From, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
	// -- Login
	LoginFailUsernameNotFound,
	LoginFailUserHasNoPwd {
		user_id: i64,
	},
	LoginFailPwdNotMatching {
		user_id: i64,
	},

	// -- CtxExtError
	#[from]
	CtxExt(middleware::mw_auth::CtxExtError),

	// -- Extractors
	ReqStampNotInReqExt,

	// -- Modules
	#[from]
	Model(model::Error),
	#[from]
	Pwd(pwd::Error),
	#[from]
	Token(token::Error),
	#[from]
	Rpc(lib_rpc_core::Error),

	// -- RpcError (deconstructed from rpc_router::Error)
	// Simple mapping for the RpcRequestParsingError. It will have the eventual id, method context.
	#[from]
	RpcRequestParsing(rpc_router::RequestParsingError),

	// When encountering `rpc_router::Error::Handler`, we deconstruct it into the appropriate concrete application error types.
	RpcLibRpc(lib_rpc_core::Error),
	// ... more types might be here, depending on our Error strategy. Usually, one per library crate is sufficient.

	// When it's `rpc_router::Error::Handler` but we did not handle the type,
	// we still capture the type name for information. This should not occur once the code is complete.
	RpcHandlerErrorUnhandled(&'static str),
	// When the `rpc_router::Error` is not a `Handler`, we can pass through the rpc_router::Error
	// as all variants contain concrete types.
	RpcRouter {
		id: Value,
		method: String,
		error: rpc_router::Error,
	},

	// -- External Modules
	#[from]
	SerdeJson(#[serde_as(as = "DisplayFromStr")] serde_json::Error),
}

// region:    --- From rpc-router::Error

/// The purpose of this `From` implementation is to extract the error types we recognize
/// from the `rpc_router`'s `RpcHandlerError` within the `rpc_router::Error::Handler`
/// and place them into the appropriate variant of our application error enum.
///
/// - The `rpc-router` provides an `RpcHandlerError` scheme to allow application RPC handlers
///   to return the errors they wish with minimal constraints.
/// - This approach requires us to "unpack" those types in our code and assign them to the correct
///   "concrete/direct" variant (not `Box<dyn Any>`...).
/// - If it's not an `rpc_router::Error::Handler` variant, then we can capture the `rpc_router::Error`
///   as it is, treating all other variants as "concrete/direct" types.
impl From<rpc_router::CallError> for Error {
	fn from(call_error: rpc_router::CallError) -> Self {
		let rpc_router::CallError { id, method, error } = call_error;
		match error {
			rpc_router::Error::Handler(mut rpc_handler_error) => {
				if let Some(lib_rpc_error) =
					rpc_handler_error.remove::<lib_rpc_core::Error>()
				{
					Error::RpcLibRpc(lib_rpc_error)
				}
				// report the unhandled error for debugging and completing code.
				else {
					let type_name = rpc_handler_error.type_name();
					warn!("Unhandled RpcHandlerError type: {type_name}");
					Error::RpcHandlerErrorUnhandled(type_name)
				}
			}
			error => Error::RpcRouter { id, method, error },
		}
	}
}

// endregion: --- From rpc-router::Error

// region:    --- Axum IntoResponse
impl IntoResponse for Error {
	fn into_response(self) -> Response {
		debug!("{:<12} - model::Error {self:?}", "INTO_RES");

		// Create a placeholder Axum reponse.
		let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

		// Insert the Error into the reponse.
		response.extensions_mut().insert(Arc::new(self));

		response
	}
}
// endregion: --- Axum IntoResponse

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

// region:    --- Client Error

/// From the root error to the http status code and ClientError
impl Error {
	pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
		use Error::*; // TODO: should change to `use web::Error as E`

		match self {
			// -- Login
			LoginFailUsernameNotFound
			| LoginFailUserHasNoPwd { .. }
			| LoginFailPwdNotMatching { .. } => {
				(StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL)
			}

			// -- Auth
			CtxExt(_) => (StatusCode::FORBIDDEN, ClientError::NO_AUTH),

			// -- Model
			Model(model::Error::EntityNotFound { entity, id }) => (
				StatusCode::BAD_REQUEST,
				ClientError::ENTITY_NOT_FOUND { entity, id: *id },
			),

			// -- Rpc
			RpcRequestParsing(req_parsing_err) => (
				StatusCode::BAD_REQUEST,
				ClientError::RPC_REQUEST_INVALID(req_parsing_err.to_string()),
			),
			RpcRouter {
				error: rpc_router::Error::MethodUnknown,
				method,
				..
			} => (
				StatusCode::BAD_REQUEST,
				ClientError::RPC_REQUEST_METHOD_UNKNOWN(format!(
					"rpc method '{method}' unknown"
				)),
			),
			RpcRouter {
				error: rpc_router::Error::ParamsParsing(params_parsing_err),
				..
			} => (
				StatusCode::BAD_REQUEST,
				ClientError::RPC_PARAMS_INVALID(params_parsing_err.to_string()),
			),
			RpcRouter {
				error: rpc_router::Error::ParamsMissingButRequested,
				method,
				..
			} => (
				StatusCode::BAD_REQUEST,
				ClientError::RPC_PARAMS_INVALID(format!(
					"Params missing. Method '{method}' requires params"
				)),
			),

			// -- Fallback.
			_ => (
				StatusCode::INTERNAL_SERVER_ERROR,
				ClientError::SERVICE_ERROR,
			),
		}
	}
}

#[derive(Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "message", content = "detail")]
#[allow(non_camel_case_types)]
pub enum ClientError {
	LOGIN_FAIL,
	NO_AUTH,
	ENTITY_NOT_FOUND { entity: &'static str, id: i64 },

	RPC_REQUEST_INVALID(String),
	RPC_REQUEST_METHOD_UNKNOWN(String),
	RPC_PARAMS_INVALID(String),

	SERVICE_ERROR,
}
// endregion: --- Client Error
