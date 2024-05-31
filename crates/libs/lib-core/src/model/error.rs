use crate::model::store::dbx;
use derive_more::From;
use lib_auth::pwd;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use sqlx::error::DatabaseError;
use std::borrow::Cow;

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize, From)]
pub enum Error {
	EntityNotFound {
		entity: &'static str,
		id: i64,
	},
	ListLimitOverMax {
		max: i64,
		actual: i64,
	},

	CountFail,

	// -- DB
	UserAlreadyExists {
		username: String,
	},
	UniqueViolation {
		table: String,
		constraint: String,
	},

	// -- ModelManager
	CantCreateModelManagerProvider(String),

	// -- Modules
	#[from]
	Pwd(pwd::Error),
	#[from]
	Dbx(dbx::Error),

	// -- Externals
	#[from]
	SeaQuery(#[serde_as(as = "DisplayFromStr")] sea_query::error::Error),

	#[from]
	ModqlIntoSea(#[serde_as(as = "DisplayFromStr")] modql::filter::IntoSeaError),
}

impl Error {
	/// This function will transform the error into a more precise variant if it is an SQLX or PGError Unique Violation.
	/// The resolver can contain a function (table_name: &str, constraint: &str) that may return a specific Error if desired.
	/// If the resolver is None, or if the resolver function returns None, it will default to Error::UniqueViolation {table, constraint}.
	pub fn resolve_unique_violation<F>(self, resolver: Option<F>) -> Self
	where
		F: FnOnce(&str, &str) -> Option<Self>,
	{
		match self.as_database_error().map(|db_error| {
			(db_error.code(), db_error.table(), db_error.constraint())
		}) {
			// "23505" => postgresql "unique violation"
			Some((Some(Cow::Borrowed("23505")), Some(table), Some(constraint))) => {
				resolver
					.and_then(|fun| fun(table, constraint))
					.unwrap_or_else(|| Error::UniqueViolation {
						table: table.to_string(),
						constraint: constraint.to_string(),
					})
			}
			_ => self,
		}
	}

	/// A convenient function to return the eventual database error (Postgres)
	/// if this Error is an SQLX Error that contains a database error.
	pub fn as_database_error(&self) -> Option<&(dyn DatabaseError + 'static)> {
		match self {
			Error::Dbx(dbx::Error::Sqlx(sqlx_error)) => {
				sqlx_error.as_database_error()
			}
			_ => None,
		}
	}
}

// region:    --- Error Boilerplate

impl core::fmt::Display for Error {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}

// endregion: --- Error Boilerplate
