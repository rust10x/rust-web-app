use crate::ctx::Ctx;
use crate::model::ModelManager;
use crate::model::{Error, Result};
use lib_base::time::now_utc;
use modql::field::{Field, Fields, HasFields};
use modql::filter::SeaFilter;
use modql::sea_utils::SIden;
use modql::ListOptions;
use sea_query::{DynIden, Expr, Iden, IntoIden, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::postgres::PgRow;
use sqlx::FromRow;

#[derive(Iden)]
pub enum CommonSpec {
	Id,
}

#[derive(Iden)]
pub enum TimestampSpec {
	Cid,
	Ctime,
	Mid,
	Mtime,
}

pub trait DbBmc {
	const TABLE: &'static str;
	const TIMESTAMP: bool;

	fn table_iden() -> DynIden {
		SIden(Self::TABLE).into_iden()
	}
}

pub async fn create<MC, E>(ctx: &Ctx, mm: &ModelManager, data: E) -> Result<i64>
where
	MC: DbBmc,
	E: HasFields,
{
	let db = mm.db();

	// -- Build query
	let mut fields = data.not_none_fields();
	add_timestamp_for_create(&mut fields, ctx.user_id());
	let (columns, values) = fields.unzip();

	let (sql, values) = Query::insert()
		.into_table(MC::table_iden())
		.columns(columns)
		.values(values)?
		.returning(Query::returning().columns([CommonSpec::Id]))
		.build_sqlx(PostgresQueryBuilder);

	// -- Exec query
	let (id,) = sqlx::query_as_with::<_, (i64,), _>(&sql, values)
		.fetch_one(db)
		.await?;

	Ok(id)
}

pub async fn get<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
where
	MC: DbBmc,
	E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
	E: HasFields,
{
	let db = mm.db();

	// -- Build query
	let (sql, values) = Query::select()
		.from(MC::table_iden())
		.columns(E::field_column_refs())
		.and_where(Expr::col(CommonSpec::Id).eq(id))
		.build_sqlx(PostgresQueryBuilder);

	// -- Exec query
	let entity = sqlx::query_as_with::<_, E, _>(&sql, values)
		.fetch_optional(db)
		.await?
		.ok_or(Error::EntityNotFound {
			entity: MC::TABLE,
			id,
		})?;

	Ok(entity)
}

pub async fn list<MC, E, F>(
	_ctx: &Ctx,
	mm: &ModelManager,
	filter: Option<F>,
	list_options: Option<ListOptions>,
) -> Result<Vec<E>>
where
	MC: DbBmc,
	F: SeaFilter,
	E: for<'r> FromRow<'r, PgRow> + Unpin + Send,
	E: HasFields,
{
	let db = mm.db();

	// -- Build the query
	let mut query = Query::select();
	query.from(MC::table_iden()).columns(E::field_column_refs());
	// condition from filter
	if let Some(cond) = filter.map(SeaFilter::into_sea_condition) {
		query.cond_where(cond);
	}
	// list options
	if let Some(list_options) = list_options {
		list_options.apply_to_sea_query(&mut query);
	}

	// -- Execute the query
	let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
	let entities = sqlx::query_as_with::<_, E, _>(&sql, values)
		.fetch_all(db)
		.await?;

	Ok(entities)
}

pub async fn update<MC, E>(
	ctx: &Ctx,
	mm: &ModelManager,
	id: i64,
	data: E,
) -> Result<()>
where
	MC: DbBmc,
	E: HasFields,
{
	let db = mm.db();

	// -- Build query
	let mut fields = data.not_none_fields();
	add_timestamp_for_update(&mut fields, ctx.user_id());
	let fields = fields.zip();

	let (sql, values) = Query::update()
		.table(MC::table_iden())
		.values(fields)
		.and_where(Expr::col(CommonSpec::Id).eq(id))
		.build_sqlx(PostgresQueryBuilder);

	// -- Execute query
	let count = sqlx::query_with(&sql, values)
		.execute(db)
		.await?
		.rows_affected();

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
	let db = mm.db();

	let (sql, values) = Query::delete()
		.from_table(MC::table_iden())
		.and_where(Expr::col(CommonSpec::Id).eq(id))
		.build_sqlx(PostgresQueryBuilder);

	let count = sqlx::query_with(&sql, values)
		.execute(db)
		.await?
		.rows_affected();

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

// region:    --- Join Info
// .column((Font::Table, Font::Name))
// .left_join(Font::Table, Expr::col((Char::Table, Char::FontId)).equals((Font::Table, Font::Id)))

// endregion: --- Join Info

// region:    --- Utils

/// Update the timestamps info for create
/// (e.g., cid, ctime, and mid, mtime will be updated with the same values)
pub fn add_timestamp_for_create(fields: &mut Fields, user_id: i64) {
	let now = now_utc();
	fields.push(Field::new(TimestampSpec::Cid.into_iden(), user_id.into()));
	fields.push(Field::new(TimestampSpec::Ctime.into_iden(), now.into()));

	fields.push(Field::new(TimestampSpec::Mid.into_iden(), user_id.into()));
	fields.push(Field::new(TimestampSpec::Mtime.into_iden(), now.into()));
}

/// Update the timestamps info only for update.
/// (.e.g., only mid, mtime will be udpated)
pub fn add_timestamp_for_update(fields: &mut Fields, user_id: i64) {
	let now = now_utc();
	fields.push(Field::new(TimestampSpec::Mid.into_iden(), user_id.into()));
	fields.push(Field::new(TimestampSpec::Mtime.into_iden(), now.into()));
}

// endregion: --- Utils
