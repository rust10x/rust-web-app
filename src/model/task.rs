use crate::ctx::Ctx;
use crate::model::ModelManager;
use crate::model::{Error, Result};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// region:    --- Task Types
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Task {
	pub id: i64,
	pub title: String,
}

#[derive(Deserialize)]
pub struct TaskForCreate {
	pub title: String,
}

#[derive(Deserialize)]
pub struct TaskForUpdate {
	pub title: Option<String>,
}
// endregion: --- Task Types

// region:    --- TaskBmc
pub struct TaskBmc;

impl TaskBmc {
	pub async fn create(
		_ctx: &Ctx,
		mm: &ModelManager,
		task_c: TaskForCreate,
	) -> Result<i64> {
		let db = mm.db();

		let (id,) = sqlx::query_as::<_, (i64,)>(
			"INSERT INTO task (title) values ($1) returning id",
		)
		.bind(task_c.title)
		.fetch_one(db)
		.await?;

		Ok(id)
	}

	pub async fn get(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<Task> {
		let db = mm.db();

		let task: Task = sqlx::query_as("SELECT * FROM task WHERE id = $1")
			.bind(id)
			.fetch_optional(db)
			.await?
			.ok_or(Error::EntityNotFound { entity: "task", id })?;

		Ok(task)
	}

	pub async fn list(_ctx: &Ctx, mm: &ModelManager) -> Result<Vec<Task>> {
		let db = mm.db();

		let tasks: Vec<Task> = sqlx::query_as("SELECT * FROM task ORDER BY id")
			.fetch_all(db)
			.await?;

		Ok(tasks)
	}

	// TODO: update

	pub async fn delete(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> {
		let db = mm.db();

		let count = sqlx::query("DELETE FROM task where id = $1")
			.bind(id)
			.execute(db)
			.await?
			.rows_affected();

		if count == 0 {
			return Err(Error::EntityNotFound { entity: "task", id });
		}

		Ok(())
	}
}
// endregion: --- TaskBmc

// region:    --- Tests
#[cfg(test)]
mod tests {
	use super::*;
	use crate::_dev_utils;
	use anyhow::Result;
	use serial_test::serial;

	#[serial]
	#[tokio::test]
	async fn test_create_ok() -> Result<()> {
		// -- Setup & Fixtures
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_title = "test_create_ok title";

		// -- Exec
		let task_c = TaskForCreate {
			title: fx_title.to_string(),
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
		_dev_utils::seed_tasks(&ctx, &mm, fx_titles).await?;

		// -- Exec
		let tasks = TaskBmc::list(&ctx, &mm).await?;

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
