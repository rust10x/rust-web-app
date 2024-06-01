use crate::ctx::Ctx;
use crate::model::base::{
	prep_fields_for_create, prep_fields_for_update, CommonIden, DbBmc,
	LIST_LIMIT_DEFAULT, LIST_LIMIT_MAX,
};
use crate::model::ModelManager;
use crate::model::{Error, Result};
use modql::field::HasSeaFields;
use modql::filter::{FilterGroups, ListOptions};
use sea_query::{Condition, Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::postgres::PgRow;
use sqlx::FromRow;
use sqlx::Row;

pub async fn create<MC, E>(ctx: &Ctx, mm: &ModelManager, data: E) -> Result<i64>
where
	MC: DbBmc,
	E: HasSeaFields,
{
	let user_id = ctx.user_id();

	// -- Extract fields (name / sea-query value expression)
	let mut fields = data.not_none_sea_fields();
	prep_fields_for_create::<MC>(&mut fields, user_id);

	// -- Build query
	let (columns, sea_values) = fields.for_sea_insert();
	let mut query = Query::insert();
	query
		.into_table(MC::table_ref())
		.columns(columns)
		.values(sea_values)?
		.returning(Query::returning().columns([CommonIden::Id]));

	// -- Exec query
	let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
	let sqlx_query = sqlx::query_as_with::<_, (i64,), _>(&sql, values);
	// NOTE: For now, we will use the _txn for all create.
	//       We could have a with_txn as function argument if perf is an issue (it should not be)
	let (id,) = mm.dbx().fetch_one(sqlx_query).await?;

	Ok(id)
}

pub async fn create_many<MC, E>(
	ctx: &Ctx,
	mm: &ModelManager,
	data: Vec<E>,
) -> Result<Vec<i64>>
where
	MC: DbBmc,
	E: HasSeaFields,
{
	let user_id = ctx.user_id();
	let mut ids = Vec::with_capacity(data.len());

	// Prepare insert query
	let mut query = Query::insert();

	for item in data {
		let mut fields = item.not_none_sea_fields();
		prep_fields_for_create::<MC>(&mut fields, user_id);
		let (columns, sea_values) = fields.for_sea_insert();

		// Append values for each item
		query
			.into_table(MC::table_ref())
			.columns(columns.clone())
			.values(sea_values)?;
	}

	query.returning(Query::returning().columns([CommonIden::Id]));

	// Execute query
	let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
	let sqlx_query = sqlx::query_as_with::<_, (i64,), _>(&sql, values);

	let rows = mm.dbx().fetch_all(sqlx_query).await?;

	for row in rows {
		let (id,): (i64,) = row;
		ids.push(id);
	}

	Ok(ids)
}

pub async fn get<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
where
	MC: DbBmc,
	E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
	E: HasSeaFields,
{
	// -- Build query
	let mut query = Query::select();
	query
		.from(MC::table_ref())
		.columns(E::sea_column_refs())
		.and_where(Expr::col(CommonIden::Id).eq(id));

	// -- Exec query
	let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
	let sqlx_query = sqlx::query_as_with::<_, E, _>(&sql, values);
	let entity =
		mm.dbx()
			.fetch_optional(sqlx_query)
			.await?
			.ok_or(Error::EntityNotFound {
				entity: MC::TABLE,
				id,
			})?;

	Ok(entity)
}

pub async fn first<MC, E, F>(
	ctx: &Ctx,
	mm: &ModelManager,
	filter: Option<F>,
	list_options: Option<ListOptions>,
) -> Result<Option<E>>
where
	MC: DbBmc,
	F: Into<FilterGroups>,
	E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
	E: HasSeaFields,
{
	let list_options = match list_options {
		Some(mut list_options) => {
			// Reset the offset/limit
			list_options.offset = None;
			list_options.limit = Some(1);

			// Don't change order_bys if not empty,
			// otherwise, set it to id (creation asc order)
			list_options.order_bys =
				list_options.order_bys.or_else(|| Some("id".into()));

			list_options
		}
		None => ListOptions {
			limit: Some(1),
			offset: None,
			order_bys: Some("id".into()), // default id asc
		},
	};

	list::<MC, E, F>(ctx, mm, filter, Some(list_options))
		.await
		.map(|item| item.into_iter().next())
}

pub async fn list<MC, E, F>(
	_ctx: &Ctx,
	mm: &ModelManager,
	filter: Option<F>,
	list_options: Option<ListOptions>,
) -> Result<Vec<E>>
where
	MC: DbBmc,
	F: Into<FilterGroups>,
	E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
	E: HasSeaFields,
{
	// -- Build the query
	let mut query = Query::select();
	query.from(MC::table_ref()).columns(E::sea_column_refs());

	// condition from filter
	if let Some(filter) = filter {
		let filters: FilterGroups = filter.into();
		let cond: Condition = filters.try_into()?;
		query.cond_where(cond);
	}
	// list options
	let list_options = compute_list_options(list_options)?;
	list_options.apply_to_sea_query(&mut query);

	// -- Execute the query
	let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

	let sqlx_query = sqlx::query_as_with::<_, E, _>(&sql, values);
	let entities = mm.dbx().fetch_all(sqlx_query).await?;

	Ok(entities)
}

pub async fn count<MC, F>(
	_ctx: &Ctx,
	mm: &ModelManager,
	filter: Option<F>,
) -> Result<i64>
where
	MC: DbBmc,
	F: Into<FilterGroups>,
{
	let db = mm.dbx().db();
	// -- Build the query
	let mut query = Query::select()
		.from(MC::table_ref())
		.expr(Expr::col(sea_query::Asterisk).count())
		.to_owned();

	// condition from filter
	if let Some(filter) = filter {
		let filters: FilterGroups = filter.into();
		let cond: Condition = filters.try_into()?;
		query.cond_where(cond);
	}

	let query_str = query.to_string(PostgresQueryBuilder);

	let result = sqlx::query(&query_str)
		.fetch_one(db)
		.await
		.map_err(|_| Error::CountFail)?;

	let count: i64 = result.try_get("count").map_err(|_| Error::CountFail)?;

	Ok(count)
}

pub async fn update<MC, E>(
	ctx: &Ctx,
	mm: &ModelManager,
	id: i64,
	data: E,
) -> Result<()>
where
	MC: DbBmc,
	E: HasSeaFields,
{
	// -- Prep Fields
	let mut fields = data.not_none_sea_fields();
	prep_fields_for_update::<MC>(&mut fields, ctx.user_id());

	// -- Build query
	let fields = fields.for_sea_update();
	let mut query = Query::update();
	query
		.table(MC::table_ref())
		.values(fields)
		.and_where(Expr::col(CommonIden::Id).eq(id));

	// -- Execute query
	let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
	let sqlx_query = sqlx::query_with(&sql, values);
	let count = mm.dbx().execute(sqlx_query).await?;

	// -- Check result
	if count == 0 {
		Err(Error::EntityNotFound {
			entity: MC::TABLE,
			id,
		})
	} else {
		Ok(())
	}
}

pub async fn delete<MC>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()>
where
	MC: DbBmc,
{
	// -- Build query
	let mut query = Query::delete();
	query
		.from_table(MC::table_ref())
		.and_where(Expr::col(CommonIden::Id).eq(id));

	// -- Execute query
	let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
	let sqlx_query = sqlx::query_with(&sql, values);
	let count = mm.dbx().execute(sqlx_query).await?;

	// -- Check result
	if count == 0 {
		Err(Error::EntityNotFound {
			entity: MC::TABLE,
			id,
		})
	} else {
		Ok(())
	}
}

pub async fn delete_many<MC>(
	_ctx: &Ctx,
	mm: &ModelManager,
	ids: Vec<i64>,
) -> Result<u64>
where
	MC: DbBmc,
{
	if ids.is_empty() {
		return Ok(0);
	}

	// -- Build query
	let mut query = Query::delete();
	query
		.from_table(MC::table_ref())
		.and_where(Expr::col(CommonIden::Id).is_in(ids.clone()));

	// -- Execute query
	let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
	let sqlx_query = sqlx::query_with(&sql, values);
	let result = mm.dbx().execute(sqlx_query).await?;

	// -- Check result
	if result as usize != ids.len() {
		Err(Error::EntityNotFound {
			entity: MC::TABLE,
			id: 0, // Using 0 because multiple IDs could be not found, you may want to improve error handling here
		})
	} else {
		Ok(result)
	}
}

pub fn compute_list_options(
	list_options: Option<ListOptions>,
) -> Result<ListOptions> {
	if let Some(mut list_options) = list_options {
		// Validate the limit.
		if let Some(limit) = list_options.limit {
			if limit > LIST_LIMIT_MAX {
				return Err(Error::ListLimitOverMax {
					max: LIST_LIMIT_MAX,
					actual: limit,
				});
			}
		}
		// Set the default limit if no limit
		else {
			list_options.limit = Some(LIST_LIMIT_DEFAULT);
		}
		Ok(list_options)
	}
	// When None, return default
	else {
		Ok(ListOptions {
			limit: Some(LIST_LIMIT_DEFAULT),
			offset: None,
			order_bys: Some("id".into()),
		})
	}
}
