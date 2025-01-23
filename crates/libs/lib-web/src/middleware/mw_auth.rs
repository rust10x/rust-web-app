use crate::error::{Error, Result};
use crate::utils::token::{set_token_cookie, AUTH_TOKEN};
use axum::body::Body;
use axum::extract::{FromRequestParts, Path, State};
use axum::http::request::Parts;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::Response;
use lazy_regex::regex_captures;
use lib_auth::token::{validate_web_token, Token};
use lib_core::ctx::Ctx;
use lib_core::model::user::{UserBmc, UserForAuth};
use lib_core::model::ModelManager;
use serde::Serialize;
use tower_cookies::{Cookie, Cookies};
use tracing::debug;

pub async fn mw_ctx_require(
	ctx: Result<CtxW>,
	req: Request<Body>,
	next: Next,
) -> Result<Response> {
	debug!("{:<12} - mw_ctx_require - {ctx:?}", "MIDDLEWARE");

	ctx?;

	Ok(next.run(req).await)
}

// IMPORTANT: This resolver must never fail, but rather capture the potential Auth error and put in in the
//            request extension as CtxExtResult.
//            This way it won't prevent downstream middleware to be executed, and will still capture the error
//            for the appropriate middleware (.e.g., mw_ctx_require which forces successful auth) or handler
//            to get the appropriate information.
pub async fn mw_ctx_resolver(
	State(mm): State<ModelManager>,
	cookies: Cookies,
	mut req: Request<Body>,
	next: Next,
) -> Response {
	debug!("{:<12} - mw_ctx_resolve", "MIDDLEWARE");
	let path = req.uri().path();
	let ctx_ext_result = ctx_resolve(mm, path, &cookies).await;

	if ctx_ext_result.is_err()
		&& !matches!(ctx_ext_result, Err(CtxExtError::TokenNotInCookie))
	{
		cookies.remove(Cookie::from(AUTH_TOKEN))
	}

	// Store the ctx_ext_result in the request extension
	// (for Ctx extractor).
	req.extensions_mut().insert(ctx_ext_result);

	next.run(req).await
}

async fn ctx_resolve(
	mm: ModelManager,
	path: &str,
	cookies: &Cookies,
) -> CtxExtResult {
	// -- Get Token String
	let token = cookies
		.get(AUTH_TOKEN)
		.map(|c| c.value().to_string())
		.ok_or(CtxExtError::TokenNotInCookie)?;

	// -- Parse Token
	let token: Token = token.parse().map_err(|_| CtxExtError::TokenWrongFormat)?;

	// -- Get UserForAuth
	let user: UserForAuth =
		UserBmc::first_by_username(&Ctx::root_ctx(), &mm, &token.ident)
			.await
			.map_err(|ex| CtxExtError::ModelAccessError(ex.to_string()))?
			.ok_or(CtxExtError::UserNotFound)?;

	// -- Validate Token
	validate_web_token(&token, user.token_salt)
		.map_err(|_| CtxExtError::FailValidate)?;

	// -- Update Token
	set_token_cookie(cookies, &user.username, user.token_salt)
		.map_err(|_| CtxExtError::CannotSetTokenCookie)?;

	// -- Create CtxExtResult

	println!("->> PATH ... {path:?}");
	let org_id = extract_org_id(path);
	println!("->> org_id {org_id:?}");
	Ctx::new(user.id, org_id)
		.map(CtxW)
		.map_err(|ex| CtxExtError::CtxCreateFail(ex.to_string()))
}

// region:    --- Support

// Returns: None if not found, the i64 if found, error if found but not number
fn extract_org_id(path: &str) -> Option<i64> {
	if let Some((_, org_number)) = regex_captures!(r#"^/api/org/(\d+)/rpc$"#, path) {
		org_number.parse::<i64>().ok()
	} else {
		None
	}
}

// endregion: --- Support

// region:    --- Ctx Extractor
#[derive(Debug, Clone)]
pub struct CtxW(pub Ctx);

impl<S: Send + Sync> FromRequestParts<S> for CtxW {
	type Rejection = Error;

	async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
		debug!("{:<12} - Ctx", "EXTRACTOR");

		parts
			.extensions
			.get::<CtxExtResult>()
			.ok_or(Error::CtxExt(CtxExtError::CtxNotInRequestExt))?
			.clone()
			.map_err(Error::CtxExt)
	}
}
// endregion: --- Ctx Extractor

// region:    --- Ctx Extractor Result/Error
type CtxExtResult = core::result::Result<CtxW, CtxExtError>;

#[derive(Clone, Serialize, Debug)]
pub enum CtxExtError {
	TokenNotInCookie,
	TokenWrongFormat,

	UserNotFound,
	ModelAccessError(String),
	FailValidate,
	CannotSetTokenCookie,

	CtxNotInRequestExt,
	CtxCreateFail(String),
}
// endregion: --- Ctx Extractor Result/Error
