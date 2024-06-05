use crate::ctx::Ctx;
use crate::generate_common_bmc_fns;
use crate::model::base::{self, DbBmc};
use crate::model::modql_utils::time_to_sea_value;
use crate::model::ModelManager;
use crate::model::Result;
use lib_utils::time::Rfc3339;
use modql::field::Fields;
use modql::filter::{FilterNodes, OpValsString, OpValsValue};
use modql::filter::{ListOptions, OpValsInt64};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sqlx::types::time::OffsetDateTime;
use sqlx::FromRow;

// region:    --- Agent Types

#[serde_as]
#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct Agent {
	pub id: i64,

	// -- Relations
	pub owner_id: i64,

	// -- Properties
	pub name: String,
	pub ai_provider: String,
	pub ai_model: String,

	// -- Timestamps
	//    (creator and last modified user_id/time)
	pub cid: i64,
	#[serde_as(as = "Rfc3339")]
	pub ctime: OffsetDateTime,
	pub mid: i64,
	#[serde_as(as = "Rfc3339")]
	pub mtime: OffsetDateTime,
}

#[derive(Fields, Deserialize)]
pub struct AgentForCreate {
	pub name: String,
}

#[derive(Fields, Deserialize)]
pub struct AgentForUpdate {
	pub name: Option<String>,
}

#[derive(FilterNodes, Default, Deserialize)]
pub struct AgentFilter {
	pub id: Option<OpValsInt64>,
	pub name: Option<OpValsString>,

	pub cid: Option<OpValsInt64>,
	#[modql(to_sea_value_fn = "time_to_sea_value")]
	pub ctime: Option<OpValsValue>,
	pub mid: Option<OpValsInt64>,
	#[modql(to_sea_value_fn = "time_to_sea_value")]
	pub mtime: Option<OpValsValue>,
}

// endregion: --- Agent Types

// region:    --- AgentBmc

pub struct AgentBmc;

impl DbBmc for AgentBmc {
	const TABLE: &'static str = "agent";

	fn has_owner_id() -> bool {
		true
	}
}

// This will generate the `impl AgentBmc {...}` with the default CRUD functions.
generate_common_bmc_fns!(
	Bmc: AgentBmc,
	Entity: Agent,
	ForCreate: AgentForCreate,
	ForUpdate: AgentForUpdate,
	Filter: AgentFilter,
);

// endregion: --- AgentBmc

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Error = Box<dyn std::error::Error>;
	type Result<T> = core::result::Result<T, Error>; // For tests.

	use super::*;
	use crate::_dev_utils::{self, clean_agents, seed_agent, seed_agents};
	use crate::model;
	use serde_json::json;
	use serial_test::serial;

	#[serial]
	#[tokio::test]
	async fn test_create_ok() -> Result<()> {
		// -- Setup & Fixtures
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_name = "test_create_ok agent 01";

		// -- Exec
		let fx_agent_c = AgentForCreate {
			name: fx_name.to_string(),
		};
		let agent_id = AgentBmc::create(&ctx, &mm, fx_agent_c).await?;

		// -- Check
		let agent = AgentBmc::get(&ctx, &mm, agent_id).await?;
		assert_eq!(agent.name, fx_name);

		// -- Clean
		let count = clean_agents(&ctx, &mm, "test_create_ok").await?;
		assert_eq!(count, 1, "Should have cleaned only 1 agent");

		Ok(())
	}

	#[serial]
	#[tokio::test]
	async fn test_create_many_ok() -> Result<()> {
		// -- Setup & Fixtures
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_name = "test_create_many_ok agent 01";

		// -- Exec
		let fx_agent_c = AgentForCreate {
			name: fx_name.to_string(),
		};
		let fx_agent_c2 = AgentForCreate {
			name: fx_name.to_string(),
		};

		let agent_ids =
			AgentBmc::create_many(&ctx, &mm, vec![fx_agent_c, fx_agent_c2]).await?;

		let agent_filter: AgentFilter = serde_json::from_value(json!(
			{
				"id": {"$in": agent_ids}
			}
		))?;

		let agents =
			AgentBmc::list(&ctx, &mm, Some(vec![agent_filter]), None).await?;

		assert_eq!(agents.len(), 2, "should have only retrieved 2 agents");

		Ok(())
	}

	#[serial]
	#[tokio::test]
	async fn test_update_ok() -> Result<()> {
		// -- Setup & Fixtures
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();

		let fx_name = "test_update_ok agent 01";
		let fx_agent_id = seed_agent(&ctx, &mm, fx_name).await?;
		let fx_name_updated = "test_update_ok agent 02 - updated";

		// -- Exec
		let fx_agent_u = AgentForUpdate {
			name: Some(fx_name_updated.to_string()),
		};
		AgentBmc::update(&ctx, &mm, fx_agent_id, fx_agent_u).await?;

		// -- Check
		let agent = AgentBmc::get(&ctx, &mm, fx_agent_id).await?;
		assert_eq!(agent.name, fx_name_updated);

		// -- Clean
		let count = clean_agents(&ctx, &mm, "test_update_ok agent").await?;
		assert_eq!(count, 1, "Should have cleaned only 1 agent");

		Ok(())
	}

	#[serial]
	#[tokio::test]
	async fn test_delete_ok() -> Result<()> {
		// -- Setup & Fixtures
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();

		let fx_name = "test_delete_ok agent 01";
		let fx_agent_id = seed_agent(&ctx, &mm, fx_name).await?;

		// -- Exec
		// check it's there
		AgentBmc::get(&ctx, &mm, fx_agent_id).await?;
		// do the delete
		AgentBmc::delete(&ctx, &mm, fx_agent_id).await?;

		// -- Check
		let res = AgentBmc::get(&ctx, &mm, fx_agent_id).await;
		assert!(
			matches!(&res, Err(model::Error::EntityNotFound { .. })),
			"should return a EntityNotFound"
		);

		Ok(())
	}
	#[serial]
	#[tokio::test]
	async fn test_delete_many_ok() -> Result<()> {
		// -- Setup & Fixtures
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_name = "test_create_ok agent 01";

		// -- Exec
		let fx_agent_c = AgentForCreate {
			name: fx_name.to_string(),
		};
		let fx_agent_c2 = AgentForCreate {
			name: fx_name.to_string(),
		};

		let agent_ids =
			AgentBmc::create_many(&ctx, &mm, vec![fx_agent_c, fx_agent_c2]).await?;

		let agent_filter: AgentFilter = serde_json::from_value(json!(
			{
				"id": {"$in": agent_ids}
			}
		))?;

		let agents =
			AgentBmc::list(&ctx, &mm, Some(vec![agent_filter]), None).await?;

		assert_eq!(agents.len(), 2, "should have only retrieved 2 agents");

		let deleted = AgentBmc::delete_many(&ctx, &mm, agent_ids).await?;

		assert_eq!(deleted, agents.len() as u64);

		Ok(())
	}

	#[serial]
	#[tokio::test]
	async fn test_first_ok() -> Result<()> {
		// -- Setup & Fixtures
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();

		let fx_agent_names = &["test_first_ok agent 01", "test_first_ok agent 02"];
		seed_agents(&ctx, &mm, fx_agent_names).await?;

		// -- Exec
		let agent_filter: AgentFilter = serde_json::from_value(json!(
			{
				"name": {"$startsWith": "test_first_ok agent"}
			}
		))?;
		let agent =
			AgentBmc::first(&ctx, &mm, Some(vec![agent_filter]), None).await?;

		// -- Check
		let agent = agent.ok_or("No Agent Returned (should have returned one")?;
		assert_eq!(agent.name, fx_agent_names[0]);

		// -- Clean
		let count = clean_agents(&ctx, &mm, "test_first_ok agent").await?;
		assert_eq!(count, 2, "Should have cleaned 2 agents");

		Ok(())
	}

	#[serial]
	#[tokio::test]
	async fn test_list_ok() -> Result<()> {
		// -- Setup & Fixtures
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();

		let fx_agent_names = &["test_list_ok agent 01", "test_list_ok agent 02"];
		seed_agents(&ctx, &mm, fx_agent_names).await?;
		let fx_asst_names = &[
			"test_list_ok asst 01",
			"test_list_ok asst 02",
			"test_list_ok asst 03",
		];
		seed_agents(&ctx, &mm, fx_asst_names).await?;

		// -- Exec
		let agent_filter: AgentFilter = serde_json::from_value(json!(
			{
				"name": {"$contains": "list_ok agent"}
			}
		))?;
		let agents =
			AgentBmc::list(&ctx, &mm, Some(vec![agent_filter]), None).await?;

		// -- Check
		assert_eq!(agents.len(), 2);
		let names = agents.iter().map(|a| &a.name).collect::<Vec<_>>();
		assert_eq!(names, fx_agent_names);

		// -- Clean
		let count = clean_agents(&ctx, &mm, "test_list_ok agent").await?;
		assert_eq!(count, 2, "Should have cleaned 2 agents");
		let count = clean_agents(&ctx, &mm, "test_list_ok asst").await?;
		assert_eq!(count, 3, "Should have cleaned 3 assts");

		Ok(())
	}
	#[serial]
	#[tokio::test]
	async fn test_count_ok() -> Result<()> {
		// -- Setup & Fixtures
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();

		let fx_agent_names = &["test_list_ok agent 01", "test_list_ok agent 02"];
		seed_agents(&ctx, &mm, fx_agent_names).await?;

		// -- Exec
		let agent_filter: AgentFilter = serde_json::from_value(json!(
			{
				"name": {"$contains": "list_ok agent"}
			}
		))?;
		let count = AgentBmc::count(&ctx, &mm, Some(vec![agent_filter])).await?;

		// -- Check
		assert_eq!(count, 2);

		// -- Clean
		let count = clean_agents(&ctx, &mm, "test_list_ok agent").await?;
		assert_eq!(count, 2, "Should have cleaned 2 agents");

		Ok(())
	}
}

// endregion: --- Tests
