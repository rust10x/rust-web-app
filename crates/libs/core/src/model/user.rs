use crate::ctx::Ctx;
use crate::model::base::{self, DbBmc};
use crate::model::ModelManager;
use crate::model::Result;
use crate::pwd::{self, ContentToHash};
use sea_query::{Expr, Iden, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use sqlb::{Fields, HasFields};
use sqlx::postgres::PgRow;
use sqlx::FromRow;
use uuid::Uuid;

// region:    --- User Types
#[derive(Clone, Fields, FromRow, Debug, Serialize)]
pub struct User {
	pub id: i64,
	pub username: String,
}

#[derive(Deserialize)]
pub struct UserForCreate {
	pub username: String,
	pub pwd_clear: String,
}

#[derive(Fields)]
pub struct UserForInsert {
	pub username: String,
}

#[derive(Clone, FromRow, Fields, Debug)]
pub struct UserForLogin {
	pub id: i64,
	pub username: String,

	// -- pwd and token info
	pub pwd: Option<String>, // hashed, #_scheme_id_#....
	pub pwd_salt: Uuid,
	pub token_salt: Uuid,
}

#[derive(Clone, FromRow, Fields, Debug)]
pub struct UserForAuth {
	pub id: i64,
	pub username: String,

	// -- token info
	pub token_salt: Uuid,
}

/// Marker trait
pub trait UserBy: HasFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}

impl UserBy for User {}
impl UserBy for UserForLogin {}
impl UserBy for UserForAuth {}
// endregion: --- User Types

// region:    --- UserSpec
// Note: Since the entity properties Iden will be given by sqlb
//       UserSpec does not have to be exhaustive, but just have the columns
//       we use in our specific code.
#[derive(Iden)]
enum UserSpec {
	Id,
	Username,
	Pwd,
}
// endregion: --- UserSpec

// region:    --- UserBmc
pub struct UserBmc;

impl DbBmc for UserBmc {
	const TABLE: &'static str = "user";
}

impl UserBmc {
	pub async fn get<E>(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E>
	where
		E: UserBy,
	{
		base::get::<Self, _>(ctx, mm, id).await
	}

	pub async fn first_by_username<E>(
		_ctx: &Ctx,
		mm: &ModelManager,
		username: &str,
	) -> Result<Option<E>>
	where
		E: UserBy,
	{
		let db = mm.db();

		// -- Build query
		let (sql, values) = Query::select()
			.from(Self::table_dyn())
			.columns(E::field_idens())
			.and_where(Expr::col(UserSpec::Username).eq(username))
			.build_sqlx(PostgresQueryBuilder);

		// -- Execute query
		let entity = sqlx::query_as_with::<_, E, _>(&sql, values)
			.fetch_optional(db)
			.await?;

		Ok(entity)
	}

	pub async fn update_pwd(
		ctx: &Ctx,
		mm: &ModelManager,
		id: i64,
		pwd_clear: &str,
	) -> Result<()> {
		let db = mm.db();

		// -- Prep password
		let user: UserForLogin = Self::get(ctx, mm, id).await?;
		let pwd = pwd::hash_pwd(&ContentToHash {
			content: pwd_clear.to_string(),
			salt: user.pwd_salt,
		})?;

		// -- Build query
		let fields = [(UserSpec::Pwd, pwd.into())];
		let (sql, values) = Query::update()
			.table(Self::table_dyn())
			.values(fields)
			.and_where(Expr::col(UserSpec::Id).eq(id))
			.build_sqlx(PostgresQueryBuilder);

		// -- Execute query
		let _count = sqlx::query_with(&sql, values)
			.execute(db)
			.await?
			.rows_affected();

		// TODO - should check that 1 was updated

		Ok(())
	}
}
// endregion: --- UserBmc

// region:    --- Tests
#[cfg(test)]
mod tests {
	use super::*;
	use crate::_dev_utils;
	use anyhow::{Context, Result};
	use serial_test::serial;

	#[serial]
	#[tokio::test]
	async fn test_first_ok_demo1() -> Result<()> {
		// -- Setup & Fixtures
		let mm = _dev_utils::init_test().await;
		let ctx = Ctx::root_ctx();
		let fx_username = "demo1";

		// -- Exec
		let user: User = UserBmc::first_by_username(&ctx, &mm, fx_username)
			.await?
			.context("Should have user 'demo1'")?;

		// -- Check
		assert_eq!(user.username, fx_username);

		Ok(())
	}
}
// endregion: --- Tests
