pub mod agent_rpc;
pub mod conv_rpc;

use rpc_router::{Router, RouterBuilder};

pub fn all_rpc_router_builder() -> RouterBuilder {
	Router::builder()
		.extend(agent_rpc::rpc_router_builder())
		.extend(conv_rpc::rpc_router_builder())
}