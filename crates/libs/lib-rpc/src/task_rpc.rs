use crate::params::ParamsList;
use crate::Result;
use crate::{ParamsForCreate, ParamsForUpdate, ParamsIded};
use lib_core::ctx::Ctx;
use lib_core::model::task::{
	Task, TaskBmc, TaskFilter, TaskForCreate, TaskForUpdate,
};
use lib_core::model::ModelManager;

pub async fn create_task(
	ctx: Ctx,
	mm: ModelManager,
	params: ParamsForCreate<TaskForCreate>,
) -> Result<Task> {
	let ParamsForCreate { data } = params;

	let id = TaskBmc::create(&ctx, &mm, data).await?;
	let task = TaskBmc::get(&ctx, &mm, id).await?;

	Ok(task)
}

pub async fn list_tasks(
	ctx: Ctx,
	mm: ModelManager,
	params: ParamsList<TaskFilter>,
) -> Result<Vec<Task>> {
	let tasks =
		TaskBmc::list(&ctx, &mm, params.filters, params.list_options).await?;

	Ok(tasks)
}

pub async fn update_task(
	ctx: Ctx,
	mm: ModelManager,
	params: ParamsForUpdate<TaskForUpdate>,
) -> Result<Task> {
	let ParamsForUpdate { id, data } = params;

	TaskBmc::update(&ctx, &mm, id, data).await?;

	let task = TaskBmc::get(&ctx, &mm, id).await?;

	Ok(task)
}

pub async fn delete_task(
	ctx: Ctx,
	mm: ModelManager,
	params: ParamsIded,
) -> Result<Task> {
	let ParamsIded { id } = params;

	let task = TaskBmc::get(&ctx, &mm, id).await?;
	TaskBmc::delete(&ctx, &mm, id).await?;

	Ok(task)
}
