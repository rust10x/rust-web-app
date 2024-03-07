use crate::ctx::Ctx;
use crate::generate_common_bmc_fns;
use crate::model::base::{self, DbBmc};
use crate::model::conv_msg::{
	ConvMsg, ConvMsgBmc, ConvMsgForCreate, ConvMsgForInsert,
};
use crate::model::modql_utils::time_to_sea_value;
use crate::model::ModelManager;
use crate::model::Result;
use lib_utils::time::Rfc3339;
use modql::field::{Fields, SeaFieldValue};
use modql::filter::{
	FilterNodes, ListOptions, OpValsInt64, OpValsString, OpValsValue,
};
use sea_query::Nullable;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sqlx::types::time::OffsetDateTime;
use sqlx::FromRow;

// region:    --- Conv Types

/// Trait to implement on entities that have a conv_id
/// This will allow Ctx to be upgraded with the corresponding conv_id for
/// future access control.
pub trait ConvScoped {
	fn conv_id(&self) -> i64;
}

#[derive(Debug, Clone, sqlx::Type, derive_more::Display, Deserialize, Serialize)]
#[sqlx(type_name = "conv_kind")]
#[cfg_attr(test, derive(PartialEq))]
pub enum ConvKind {
	OwnerOnly,
	MultiUsers,
}

/// Note: Manual implementation.
///       Required for a modql::field::Fields
impl From<ConvKind> for sea_query::Value {
	fn from(val: ConvKind) -> Self {
		val.to_string().into()
	}
}

/// Note: Manual implementation.
///       This is required for sea::query in case of None.
///       However, in this codebase, we utilize the modql not_none_field,
///       so this will be disregarded anyway.
///       Nonetheless, it's still necessary for compilation.
impl Nullable for ConvKind {
	fn null() -> sea_query::Value {
		ConvKind::OwnerOnly.into()
	}
}

/// Note: Here we derive from modql `SeaFieldValue` which implements
///       the `From<ConvState> for sea_query::Value` and the
///       `sea_query::value::Nullable for ConvState`
///       See the `ConvKind` for the manual implementation.
///       
#[derive(
	Debug,
	Clone,
	sqlx::Type,
	SeaFieldValue,
	derive_more::Display,
	Deserialize,
	Serialize,
)]
#[sqlx(type_name = "conv_state")]
pub enum ConvState {
	Active,
	Archived,
}

#[serde_as]
#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct Conv {
	pub id: i64,

	// -- Relations
	pub agent_id: i64,
	pub owner_id: i64,

	// -- Properties
	pub title: Option<String>,
	pub kind: ConvKind,
	pub state: ConvState,

	// -- Timestamps
	// creator user_id and time
	pub cid: i64,
	#[serde_as(as = "Rfc3339")]
	pub ctime: OffsetDateTime,
	// last modifier user_id and time
	pub mid: i64,
	#[serde_as(as = "Rfc3339")]
	pub mtime: OffsetDateTime,
}

#[derive(Fields, Deserialize, Default)]
pub struct ConvForCreate {
	pub agent_id: i64,

	pub title: Option<String>,

	#[field(cast_as = "conv_kind")]
	pub kind: Option<ConvKind>,
}

#[derive(Fields, Deserialize, Default)]
pub struct ConvForUpdate {
	pub owner_id: Option<i64>,
	pub title: Option<String>,
	pub closed: Option<bool>,
	#[field(cast_as = "conv_state")]
	pub state: Option<ConvState>,
}

#[derive(FilterNodes, Deserialize, Default, Debug)]
pub struct ConvFilter {
	pub id: Option<OpValsInt64>,

	pub owner_id: Option<OpValsInt64>,
	pub agent_id: Option<OpValsInt64>,

	#[modql(cast_as = "conv_kind")]
	pub kind: Option<OpValsString>,

	pub title: Option<OpValsString>,

	pub cid: Option<OpValsInt64>,
	#[modql(to_sea_value_fn = "time_to_sea_value")]
	pub ctime: Option<OpValsValue>,
	pub mid: Option<OpValsInt64>,
	#[modql(to_sea_value_fn = "time_to_sea_value")]
	pub mtime: Option<OpValsValue>,
}

// endregion: --- Conv Types

// region:    --- ConvBmc

pub struct ConvBmc;

impl DbBmc for ConvBmc {
	const TABLE: &'static str = "conv";

	fn has_owner_id() -> bool {
		true
	}
}

// This will generate the `impl ConvBmc {...}` with the default CRUD functions.
generate_common_bmc_fns!(
	Bmc: ConvBmc,
	Entity: Conv,
	ForCreate: ConvForCreate,
	ForUpdate: ConvForUpdate,
	Filter: ConvFilter,
);

// Additional ConvBmc methods to manage the `ConvMsg` constructs.
impl ConvBmc {
	/// Add a `ConvMsg` to a `Conv`
	///
	// For access constrol, we will add:
	// #[ctx_add(conv, space)]
	// #[requires_privilege_any_of("og:FullAccess", "sp:FullAccess", "conv@owner_id" "conv:AddMsg")]
	pub async fn add_msg(
		ctx: &Ctx,
		mm: &ModelManager,
		msg_c: ConvMsgForCreate,
	) -> Result<i64> {
		let msg_i = ConvMsgForInsert::from_msg_for_create(ctx.user_id(), msg_c);
		let conv_msg_id = base::create::<ConvMsgBmc, _>(ctx, mm, msg_i).await?;

		Ok(conv_msg_id)
	}

	/// NOTE: The current strategy is to not require conv_id, but we will check
	///       that user have `conv:ReadMsg` privilege on correponding conv (post base::get).
	pub async fn get_msg(
		ctx: &Ctx,
		mm: &ModelManager,
		msg_id: i64,
	) -> Result<ConvMsg> {
		let conv_msg: ConvMsg = base::get::<ConvMsgBmc, _>(ctx, mm, msg_id).await?;

		// TODO: Validate conv_msg is with ctx.conv_id
		//       let _ctx = ctx.add_conv_id(conv_msg.conv_id());
		//       assert_privileges(&ctx, &mm, &["conv@owner_id", "conv:ReadMsg"]);

		Ok(conv_msg)
	}
}

// endregion: --- ConvBmc

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Error = Box<dyn std::error::Error>;
	type Result<T> = core::result::Result<T, Error>; // For tests.

	use super::*;
	use crate::_dev_utils::{self, seed_agent};
	use crate::ctx::Ctx;
	use crate::model::agent::AgentBmc;
	use modql::filter::OpValString;
	use serial_test::serial;

	#[serial]
	#[tokio::test]
	async fn test_create_ok() -> Result<()> {
		// -- Setup & Fixtures
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_title = "test_create_ok conv 01";
		let fx_kind = ConvKind::MultiUsers;
		let agent_id = seed_agent(&ctx, &mm, "test_create_ok conv agent 01").await?;

		// -- Exec
		let conv_id = ConvBmc::create(
			&ctx,
			&mm,
			ConvForCreate {
				agent_id,
				title: Some(fx_title.to_string()),
				kind: Some(fx_kind.clone()),
			},
		)
		.await?;

		// -- Check
		let conv: Conv = ConvBmc::get(&ctx, &mm, conv_id).await?;
		assert_eq!(&conv.kind, &fx_kind);
		assert_eq!(conv.title.ok_or("conv should have title")?, fx_title);

		// -- Clean
		ConvBmc::delete(&ctx, &mm, conv_id).await?;
		AgentBmc::delete(&ctx, &mm, agent_id).await?;

		Ok(())
	}

	#[serial]
	#[tokio::test]
	async fn test_list_ok() -> Result<()> {
		// -- Setup & Fixtures
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_title_prefix = "test_list_ok conv - ";
		let agent_id = seed_agent(&ctx, &mm, "test_create_ok conv agent 01").await?;

		for i in 1..=6 {
			let kind = if i <= 3 {
				ConvKind::OwnerOnly
			} else {
				ConvKind::MultiUsers
			};

			let _conv_id = ConvBmc::create(
				&ctx,
				&mm,
				ConvForCreate {
					agent_id,
					title: Some(format!("{fx_title_prefix}{:<02}", i)),
					kind: Some(kind),
				},
			)
			.await?;
		}

		// -- Exec
		let convs = ConvBmc::list(
			&ctx,
			&mm,
			Some(vec![ConvFilter {
				agent_id: Some(agent_id.into()),

				kind: Some(OpValString::In(vec!["MultiUsers".to_string()]).into()),
				// or
				// kind: Some(OpValString::Eq("MultiUsers".to_string()).into()),
				..Default::default()
			}]),
			None,
		)
		.await?;

		// -- Check
		// extract the 04, 05, 06 parts of the tiles
		let num_parts = convs
			.iter()
			.filter_map(|c| c.title.as_ref().and_then(|s| s.split("- ").nth(1)))
			.collect::<Vec<&str>>();
		assert_eq!(num_parts, &["04", "05", "06"]);

		// -- Clean
		// This should delete cascade
		AgentBmc::delete(&ctx, &mm, agent_id).await?;

		Ok(())
	}
}

// endregion: --- Tests
