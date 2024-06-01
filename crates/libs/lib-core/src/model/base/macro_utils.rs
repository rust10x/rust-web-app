/// Convenience macro rules to generate default CRUD functions for a Bmc/Entity.
/// Note: If custom functionality is required, use the code below as foundational
///       code for the custom implementations.
#[macro_export]
macro_rules! generate_common_bmc_fns {
	(
		Bmc: $struct_name:ident,
		Entity: $entity:ty,
		$(ForCreate: $for_create:ty,)?
		$(ForUpdate: $for_update:ty,)?
		$(Filter: $filter:ty,)?
	) => {
		impl $struct_name {
			$(
				pub async fn create(
					ctx: &Ctx,
					mm: &ModelManager,
					entity_c: $for_create,
				) -> Result<i64> {
					base::create::<Self, _>(ctx, mm, entity_c).await
				}

				pub async fn create_many(
					ctx: &Ctx,
					mm: &ModelManager,
					entity_c: Vec<$for_create>,
				) -> Result<Vec<i64>> {
					base::create_many::<Self, _>(ctx, mm, entity_c).await
				}
			)?

				pub async fn get(
					ctx: &Ctx,
					mm: &ModelManager,
					id: i64,
				) -> Result<$entity> {
					base::get::<Self, _>(ctx, mm, id).await
				}

			$(
				pub async fn first(
					ctx: &Ctx,
					mm: &ModelManager,
					filter: Option<Vec<$filter>>,
					list_options: Option<ListOptions>,
				) -> Result<Option<$entity>> {
					base::first::<Self, _, _>(ctx, mm, filter, list_options).await
				}

				pub async fn list(
					ctx: &Ctx,
					mm: &ModelManager,
					filter: Option<Vec<$filter>>,
					list_options: Option<ListOptions>,
				) -> Result<Vec<$entity>> {
					base::list::<Self, _, _>(ctx, mm, filter, list_options).await
				}

				pub async fn count(
					ctx: &Ctx,
					mm: &ModelManager,
					filter: Option<Vec<$filter>>,
				) -> Result<i64> {
					base::count::<Self, _>(ctx, mm, filter).await
				}
			)?

			$(
				pub async fn update(
					ctx: &Ctx,
					mm: &ModelManager,
					id: i64,
					entity_u: $for_update,
				) -> Result<()> {
					base::update::<Self, _>(ctx, mm, id, entity_u).await
				}
			)?

				pub async fn delete(
					ctx: &Ctx,
					mm: &ModelManager,
					id: i64,
				) -> Result<()> {
					base::delete::<Self>(ctx, mm, id).await
				}

				pub async fn delete_many(
					ctx: &Ctx,
					mm: &ModelManager,
					ids: Vec<i64>,
				) -> Result<u64> {
					base::delete_many::<Self>(ctx, mm, ids).await
				}
		}
	};
}
