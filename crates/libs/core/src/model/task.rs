use crate::ctx::Ctx;
use crate::model::base::{self, DbBmc};
use crate::model::ModelManager;
use crate::model::Result;
use lib_base::time::Rfc3339;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};
use sqlb::{Fields, HasFields};
use sqlx::types::time::OffsetDateTime;
use sqlx::FromRow;

// region:    --- Task Types
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct Task {
	pub id: i64,

	pub project_id: i64,

	pub title: String,

	// -- Timestamps
	pub cid: i64,
	#[serde_as(as = "Rfc3339")]
	pub ctime: OffsetDateTime,
	pub mid: i64,
	#[serde_as(as = "Rfc3339")]
	pub mtime: OffsetDateTime,
}

#[derive(Fields, Deserialize)]
pub struct TaskForCreate {
	pub title: String,
	pub project_id: i64,
}

#[derive(Fields, Deserialize)]
pub struct TaskForUpdate {
	pub title: Option<String>,
}

#[derive(Fields, Deserialize)]
pub struct TaskFilter {
	project_id: Option<i64>,
}
// endregion: --- Task Types

// region:    --- TaskBmc
pub struct TaskBmc;

impl DbBmc for TaskBmc {
	const TABLE: &'static str = "task";
	const TIMESTAMP: bool = true;
}

impl TaskBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		task_c: TaskForCreate,
	) -> Result<i64> {
		base::create::<Self, _>(ctx, mm, task_c).await
	}

	pub async fn get(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<Task> {
		base::get::<Self, _>(ctx, mm, id).await
	}

	pub async fn list(
		ctx: &Ctx,
		mm: &ModelManager,
		filter: Option<TaskFilter>,
	) -> Result<Vec<Task>> {
		let simple_filter = filter.map(|f| f.not_none_fields());
		base::list::<Self, _>(ctx, mm, simple_filter).await
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: i64,
		task_u: TaskForUpdate,
	) -> Result<()> {
		base::update::<Self, _>(ctx, mm, id, task_u).await
	}

	pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> {
		base::delete::<Self>(ctx, mm, id).await
	}
}
// endregion: --- TaskBmc

// region:    --- Tests
#[cfg(test)]
mod tests {
	use super::*;
	use crate::_dev_utils;
	use crate::model::Error;
	use anyhow::Result;
	use serial_test::serial;

	#[serial]
	#[tokio::test]
	async fn test_create_ok() -> Result<()> {
		// -- Setup & Fixtures
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_title = "test_create_ok title";
		let project_id =
			_dev_utils::seed_project(&ctx, &mm, "project for task test_create_ok")
				.await?;

		// -- Exec
		let task_c = TaskForCreate {
			title: fx_title.to_string(),
			project_id,
		};
		let id = TaskBmc::create(&ctx, &mm, task_c).await?;

		// -- Check
		let task = TaskBmc::get(&ctx, &mm, id).await?;
		assert_eq!(task.title, fx_title);

		// -- Clean
		TaskBmc::delete(&ctx, &mm, id).await?;

		Ok(())
	}

	#[serial]
	#[tokio::test]
	async fn test_get_err_not_found() -> Result<()> {
		// -- Setup & Fixtures
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_id = 100;

		// -- Exec
		let res = TaskBmc::get(&ctx, &mm, fx_id).await;

		// -- Check
		assert!(
			matches!(
				res,
				Err(Error::EntityNotFound {
					entity: "task",
					id: 100
				})
			),
			"EntityNotFound not matching"
		);

		Ok(())
	}

	#[serial]
	#[tokio::test]
	async fn test_list_ok() -> Result<()> {
		// -- Setup & Fixtures
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_titles = &["test_list_ok-task 01", "test_list_ok-task 02"];
		let fx_project_id =
			_dev_utils::seed_project(&ctx, &mm, "project for task test_list_ok")
				.await?;
		_dev_utils::seed_tasks(&ctx, &mm, fx_project_id, fx_titles).await?;

		// -- Exec
		let tasks = TaskBmc::list(&ctx, &mm, None).await?;

		// -- Check
		let tasks: Vec<Task> = tasks
			.into_iter()
			.filter(|t| t.title.starts_with("test_list_ok-task"))
			.collect();
		assert_eq!(tasks.len(), 2, "number of seeded tasks.");

		// -- Clean
		for task in tasks.iter() {
			TaskBmc::delete(&ctx, &mm, task.id).await?;
		}

		Ok(())
	}

	#[serial]
	#[tokio::test]
	async fn test_update_ok() -> Result<()> {
		// -- Setup & Fixtures
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_title = "test_update_ok - task 01";
		let fx_title_new = "test_update_ok - task 01 - new";
		let fx_project_id =
			_dev_utils::seed_project(&ctx, &mm, "project for task test_list_ok")
				.await?;

		let fx_task = _dev_utils::seed_tasks(&ctx, &mm, fx_project_id, &[fx_title])
			.await?
			.remove(0);

		// -- Exec
		TaskBmc::update(
			&ctx,
			&mm,
			fx_task.id,
			TaskForUpdate {
				title: Some(fx_title_new.to_string()),
			},
		)
		.await?;

		// -- Check
		let task = TaskBmc::get(&ctx, &mm, fx_task.id).await?;
		assert_eq!(task.title, fx_title_new);

		Ok(())
	}

	#[serial]
	#[tokio::test]
	async fn test_delete_err_not_found() -> Result<()> {
		// -- Setup & Fixtures
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_id = 100;

		// -- Exec
		let res = TaskBmc::delete(&ctx, &mm, fx_id).await;

		// -- Check
		assert!(
			matches!(
				res,
				Err(Error::EntityNotFound {
					entity: "task",
					id: 100
				})
			),
			"EntityNotFound not matching"
		);

		Ok(())
	}
}
// endregion: --- Tests
