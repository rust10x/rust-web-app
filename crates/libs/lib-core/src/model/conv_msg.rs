use crate::model::base::DbBmc;
use crate::model::conv::ConvScoped;
use crate::model::modql_utils::time_to_sea_value;
use lib_utils::time::Rfc3339;
use modql::field::Fields;
use modql::filter::{FilterNodes, OpValsInt64, OpValsString, OpValsValue};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sqlx::FromRow;
use time::OffsetDateTime;

// region:    --- Types

#[serde_as]
#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct ConvMsg {
	pub id: i64,

	// -- FK
	pub conv_id: i64,
	pub user_id: i64,

	// -- Properties
	pub content: String,

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

impl ConvScoped for ConvMsg {
	fn conv_id(&self) -> i64 {
		self.conv_id
	}
}

#[derive(Fields, Deserialize)]
pub struct ConvMsgForCreate {
	pub conv_id: i64,
	pub content: String,
}

impl ConvScoped for ConvMsgForCreate {
	fn conv_id(&self) -> i64 {
		self.conv_id
	}
}

/// ConvMsg for Insert, which is derived from the public `ConvMsgForCreate`.
///
/// Notes:
///   - When `...ForCreate` requires additional information for insertion into the DB, the pattern
///     is to create a `...ForInsert` type, visible only in the model layer.
///   - This approach maintains a simple and ergonomic public API while ensuring
///     strong typing for database insertion.
///   - Exceptions apply to lower-level attributes like cid, ctime, mid, mtime, and owner_id,
///     which can be set directly through the base:: functions or some utilities. There's not
///     significant value in introducing `...ForInsert` types for all entities just for these
///     common, low-level database properties.
#[derive(Fields, Deserialize)]
pub(in crate::model) struct ConvMsgForInsert {
	pub conv_id: i64,
	pub user_id: i64,
	pub content: String,
}

impl ConvMsgForInsert {
	pub fn from_msg_for_create(user_id: i64, msg_c: ConvMsgForCreate) -> Self {
		Self {
			conv_id: msg_c.conv_id,
			user_id,
			content: msg_c.content,
		}
	}
}

#[derive(Fields, Deserialize, Default)]
pub struct ConvMsgForUpdate {
	pub conv_id: i64,
	pub content: Option<String>,
}

impl ConvScoped for ConvMsgForUpdate {
	fn conv_id(&self) -> i64 {
		self.conv_id
	}
}

#[derive(FilterNodes, Deserialize, Default, Debug)]
pub struct ConvMsgFilter {
	id: Option<OpValsInt64>,

	conv_id: Option<OpValsInt64>,
	content: Option<OpValsString>,

	cid: Option<OpValsInt64>,
	#[modql(to_sea_value_fn = "time_to_sea_value")]
	ctime: Option<OpValsValue>,
	mid: Option<OpValsInt64>,
	#[modql(to_sea_value_fn = "time_to_sea_value")]
	mtime: Option<OpValsValue>,
}

// endregion: --- Types

// region:    --- ConvMsgBmc

pub struct ConvMsgBmc;

impl DbBmc for ConvMsgBmc {
	const TABLE: &'static str = "conv_msg";
}

// Note: The strategy here is to not implement `ConvMsgBmc` CRUD functions,
//       as they will be managed directly from the `ConvBmc` construct,
//       for instance with `ConvBmc::add_msg`.
//       This is because `ConvMsg` is an leaf entity better managed by its container `ConvBmc`.

// endregion: --- ConvMsgBmc
