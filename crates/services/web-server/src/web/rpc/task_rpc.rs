use crate::web::rpc::{ParamsForCreate, ParamsForUpdate, ParamsIded, ParamsList};
use crate::web::Result;
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
	params: Option<ParamsList<TaskFilter>>,
) -> Result<Vec<Task>> {
	let (filter, list_options) = params.map(|p| (p.filter, p.list_options)).unzip();
	let tasks =
		TaskBmc::list(&ctx, &mm, filter.flatten(), list_options.flatten()).await?;

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
