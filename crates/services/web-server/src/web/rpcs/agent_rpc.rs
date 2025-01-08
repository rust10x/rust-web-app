use lib_rpc_core::prelude::*;
use lib_core::model::agent::{
	Agent, AgentBmc, AgentFilter, AgentForCreate, AgentForUpdate,
};

pub fn rpc_router_builder() -> RouterBuilder {
	router_builder!(
		// Same as RpcRouter::new().add...
		create_agent,
		get_agent,
		list_agents,
		update_agent,
		delete_agent,
	)
}

generate_common_rpc_fns!(
	Bmc: AgentBmc,
	Entity: Agent,
	ForCreate: AgentForCreate,
	ForUpdate: AgentForUpdate,
	Filter: AgentFilter,
	Suffix: agent
);
