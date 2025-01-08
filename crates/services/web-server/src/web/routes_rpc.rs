use crate::web::rpcs::all_rpc_router_builder;
use axum::routing::post;
use axum::Router;
use lib_core::model::ModelManager;
use lib_web::handlers::handlers_rpc;

///  Build the Axum router for '/api/rpc'
/// Note: This will build the `rpc-router::Router` that will be used by the
///       rpc_axum_handler
pub fn routes(mm: ModelManager) -> Router {
	// Build the combined Rpc Router (from `rpc-router` crate)
	let rpc_router = all_rpc_router_builder()
		// Add the common resources for all rpc calls
		.append_resource(mm)
		.build();

	// Build the Axum Router for '/rpc'
	Router::new()
		.route("/rpc", post(handlers_rpc::rpc_axum_handler))
		.with_state(rpc_router)
}
