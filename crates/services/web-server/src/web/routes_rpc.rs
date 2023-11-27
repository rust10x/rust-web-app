use crate::web::mw_auth::CtxW;
use crate::web::Result;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use lib_core::ctx::Ctx;
use lib_core::model::ModelManager;
use lib_rpc::{exec_rpc, RpcRequest};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::debug;

pub fn routes(mm: ModelManager) -> Router {
	Router::new()
		.route("/rpc", post(rpc_handler))
		.with_state(mm)
}

/// RPC basic information containing the rpc request
/// id and method for additional logging purposes.
#[derive(Debug)]
pub struct RpcInfo {
	pub id: Option<Value>,
	pub method: String,
}

async fn rpc_handler(
	State(mm): State<ModelManager>,
	ctx: CtxW,
	Json(rpc_req): Json<RpcRequest>,
) -> Response {
	let ctx = ctx.0;
	// -- Create the RPC Info to be set to the response.extensions.
	let rpc_info = RpcInfo {
		id: rpc_req.id.clone(),
		method: rpc_req.method.clone(),
	};

	// -- Exec & Store RpcInfo in response.
	let mut res = _rpc_handler(ctx, mm, rpc_req).await.into_response();
	res.extensions_mut().insert(Arc::new(rpc_info));

	res
}

async fn _rpc_handler(
	ctx: Ctx,
	mm: ModelManager,
	rpc_req: RpcRequest,
) -> Result<Json<Value>> {
	let rpc_method = rpc_req.method.clone();
	let rpc_id = rpc_req.id.clone();

	debug!("{:<12} - _rpc_handler - method: {rpc_method}", "HANDLER");

	let result = exec_rpc(ctx, mm, rpc_req).await?;

	let body_response = json!({
		"id": rpc_id,
		"result": result
	});

	Ok(Json(body_response))
}
