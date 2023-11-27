// region:    --- Modules

mod error;
mod params;
mod task_rpc;

pub use self::error::{Error, Result};
use params::*;

use lib_core::ctx::Ctx;
use lib_core::model::ModelManager;
use serde::Deserialize;
use serde_json::{from_value, to_value, Value};
use task_rpc::{create_task, delete_task, list_tasks, update_task};

// endregion: --- Modules

// region:    --- RPC Types

/// The raw JSON-RPC request object, serving as the foundation for RPC routing.
#[derive(Deserialize)]
pub struct RpcRequest {
	pub id: Option<Value>,
	pub method: String,
	pub params: Option<Value>,
}

macro_rules! exec_rpc_fn {
	// With Params
	($rpc_fn:expr, $ctx:expr, $mm:expr, $rpc_params:expr) => {{
		let rpc_fn_name = stringify!($rpc_fn);
		let params = $rpc_params.ok_or(Error::RpcMissingParams {
			rpc_method: rpc_fn_name.to_string(),
		})?;
		let params = from_value(params).map_err(|_| Error::RpcFailJsonParams {
			rpc_method: rpc_fn_name.to_string(),
		})?;
		$rpc_fn($ctx, $mm, params).await.map(to_value)??
	}};

	// Without Params
	($rpc_fn:expr, $ctx:expr, $mm:expr) => {
		$rpc_fn($ctx, $mm).await.map(to_value)??
	};
}

pub async fn exec_rpc(
	ctx: Ctx,
	mm: ModelManager,
	rpc_req: RpcRequest,
) -> Result<Value> {
	let rpc_method = rpc_req.method;
	let rpc_params = rpc_req.params;

	// -- Exec & Store RpcInfo in response.
	let result_json: Value = match rpc_method.as_str() {
		// -- Task RPC methods.
		"create_task" => exec_rpc_fn!(create_task, ctx, mm, rpc_params),
		"list_tasks" => exec_rpc_fn!(list_tasks, ctx, mm, rpc_params),
		"update_task" => exec_rpc_fn!(update_task, ctx, mm, rpc_params),
		"delete_task" => exec_rpc_fn!(delete_task, ctx, mm, rpc_params),

		// -- Fallback as Err.
		_ => return Err(Error::RpcMethodUnknown(rpc_method)),
	};

	Ok(result_json)
}

// endregion: --- RPC Types
