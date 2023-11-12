//! Base constructs for the typed RPC Params that will be used in their respective
//! rpc handler functions (e.g., `task_rpc::create_task` and `task_rpc::list_tasks`).
//!
//! Most of these base constructs use generics for their respective data elements, allowing
//! each rpc handler function to receive the exact desired type.
//!

use modql::filter::ListOptions;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_with::{serde_as, OneOrMany};

#[derive(Deserialize)]
pub struct ParamsForCreate<D> {
	pub data: D,
}

#[derive(Deserialize)]
pub struct ParamsForUpdate<D> {
	pub id: i64,
	pub data: D,
}

#[derive(Deserialize)]
pub struct ParamsIded {
	pub id: i64,
}

#[serde_as]
#[derive(Deserialize)]
pub struct ParamsList<F>
where
	F: DeserializeOwned,
{
	#[serde_as(deserialize_as = "Option<OneOrMany<_>>")]
	pub filters: Option<Vec<F>>,
	pub list_options: Option<ListOptions>,
}
