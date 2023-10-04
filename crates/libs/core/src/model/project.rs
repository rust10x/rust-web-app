use crate::ctx::Ctx;
use crate::model::base::{self, DbBmc};
use crate::model::ModelManager;
use crate::model::Result;
use lib_base::time::Rfc3339;
use modql::field::Fields;
use modql::filter::{FilterNodes, OpValsString};
use modql::ListOptions;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sqlx::types::time::OffsetDateTime;
use sqlx::FromRow;

// region:    --- Project Types
#[serde_as]
#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct Project {
	pub id: i64,
	pub name: String,
	pub owner_id: i64,

	// -- Timestamps
	pub cid: i64,
	#[serde_as(as = "Rfc3339")]
	pub ctime: OffsetDateTime,
	pub mid: i64,
	#[serde_as(as = "Rfc3339")]
	pub mtime: OffsetDateTime,
}

#[derive(Deserialize)]
pub struct ProjectForCreate {
	pub name: String,
}

#[derive(Fields, Deserialize)]
pub struct ProjectForUpdate {
	pub name: Option<String>,
	pub owner_id: Option<i64>,
}

/// The `ProjectForCreate` contains all necessary properties
/// for a database insert.
/// NOTE: In this design, `project.owner_id` is intrinsic to the
///       `ProjectCreate` action, and should not be exposed to the API.
///       It must be respected in rights by referencing the user initiating the action.
///       Hence, in this scenario, we differentiate between `ProjectForCreate` (the public data structure)
///       and `ProjectForCreateInner` (the representation of the data to be executed, i.e., inserted).
/// (e.g., `owner_id` which is a db required field)
#[derive(Fields)]
struct ProjectForCreateInner {
	pub name: String,
	pub owner_id: i64,
}

#[derive(FilterNodes, Default, Deserialize)]
pub struct ProjectFilter {
	name: Option<OpValsString>,
}
// endregion: --- Project Types

// region:    --- Project TypesBmc
pub struct ProjectBmc;

impl DbBmc for ProjectBmc {
	const TABLE: &'static str = "project";
	const TIMESTAMP: bool = true;
}

impl ProjectBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		project_c: ProjectForCreate,
	) -> Result<i64> {
		let project_c = ProjectForCreateInner {
			name: project_c.name,
			owner_id: ctx.user_id(),
		};
		base::create::<Self, _>(ctx, mm, project_c).await
	}

	pub async fn get(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<Project> {
		base::get::<Self, _>(ctx, mm, id).await
	}

	pub async fn list(
		ctx: &Ctx,
		mm: &ModelManager,
		filter: Option<ProjectFilter>,
		list_options: Option<ListOptions>,
	) -> Result<Vec<Project>> {
		base::list::<Self, _, _>(ctx, mm, filter, list_options).await
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: i64,
		project_u: ProjectForUpdate,
	) -> Result<()> {
		base::update::<Self, _>(ctx, mm, id, project_u).await
	}

	pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> {
		base::delete::<Self>(ctx, mm, id).await
	}
}
// endregion: --- Project TypesBmc
