use crate::router::RpcRouter;

pub mod agent_rpc;
pub mod conv_rpc;
mod macro_utils;
mod prelude;

pub fn all_rpc_router() -> RpcRouter {
	RpcRouter::new()
		.extend(agent_rpc::rpc_router())
		.extend(conv_rpc::rpc_router())
}
