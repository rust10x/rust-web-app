use crate::router::RpcRouter;
use crate::rpc_router;
use crate::Result;
use crate::{ParamsForCreate, ParamsForUpdate, ParamsIded, ParamsList};
use lib_core::ctx::Ctx;
use lib_core::model::project::{
	Project, ProjectBmc, ProjectFilter, ProjectForCreate, ProjectForUpdate,
};
use lib_core::model::ModelManager;

pub fn rpc_router() -> RpcRouter {
	rpc_router!(
		// Same as RpcRouter::new().add...
		create_project,
		list_projects,
		update_project,
		delete_project,
	)
}

pub async fn create_project(
	ctx: Ctx,
	mm: ModelManager,
	params: ParamsForCreate<ProjectForCreate>,
) -> Result<Project> {
	let ParamsForCreate { data } = params;

	let id = ProjectBmc::create(&ctx, &mm, data).await?;
	let project = ProjectBmc::get(&ctx, &mm, id).await?;

	Ok(project)
}

pub async fn list_projects(
	ctx: Ctx,
	mm: ModelManager,
	params: ParamsList<ProjectFilter>,
) -> Result<Vec<Project>> {
	let projects =
		ProjectBmc::list(&ctx, &mm, params.filters, params.list_options).await?;

	Ok(projects)
}

pub async fn update_project(
	ctx: Ctx,
	mm: ModelManager,
	params: ParamsForUpdate<ProjectForUpdate>,
) -> Result<Project> {
	let ParamsForUpdate { id, data } = params;

	ProjectBmc::update(&ctx, &mm, id, data).await?;

	let project = ProjectBmc::get(&ctx, &mm, id).await?;

	Ok(project)
}

pub async fn delete_project(
	ctx: Ctx,
	mm: ModelManager,
	params: ParamsIded,
) -> Result<Project> {
	let ParamsIded { id } = params;

	let project = ProjectBmc::get(&ctx, &mm, id).await?;
	ProjectBmc::delete(&ctx, &mm, id).await?;

	Ok(project)
}
