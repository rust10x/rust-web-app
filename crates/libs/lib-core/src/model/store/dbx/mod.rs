// region:    --- Modules

mod error;

pub use error::{Error, Result};

use crate::model::store::Db;
use sqlx::query::{Query, QueryAs};
use sqlx::{FromRow, IntoArguments, Pool, Postgres, Transaction};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tokio::sync::Mutex;

// endregion: --- Modules

#[derive(Debug, Clone)]
pub struct Dbx {
	db_pool: Db,
	txn_holder: Arc<Mutex<Option<TxnHolder>>>,
	with_txn: bool,
}

impl Dbx {
	pub fn new(db_pool: Db, with_txn: bool) -> Result<Self> {
		Ok(Dbx {
			db_pool,
			txn_holder: Arc::default(),
			with_txn,
		})
	}
}

#[derive(Debug)]
struct TxnHolder {
	txn: Transaction<'static, Postgres>,
	counter: i32,
}

impl TxnHolder {
	fn new(txn: Transaction<'static, Postgres>) -> Self {
		TxnHolder { txn, counter: 1 }
	}

	fn inc(&mut self) {
		self.counter += 1;
	}

	fn dec(&mut self) -> i32 {
		self.counter -= 1;
		self.counter
	}
}

impl Deref for TxnHolder {
	type Target = Transaction<'static, Postgres>;

	fn deref(&self) -> &Self::Target {
		&self.txn
	}
}

impl DerefMut for TxnHolder {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.txn
	}
}

impl Dbx {
	pub async fn begin_txn(&self) -> Result<()> {
		if !self.with_txn {
			return Err(Error::CannotBeginTxnWithTxnFalse);
		}

		let mut txh_g = self.txn_holder.lock().await;
		// If we already have a tx holder, then, we increment
		if let Some(txh) = txh_g.as_mut() {
			txh.inc();
		}
		// If not, we create one with a new transaction
		else {
			let transaction = self.db_pool.begin().await?;
			let _ = txh_g.insert(TxnHolder::new(transaction));
		}

		Ok(())
	}

	pub async fn rollback_txn(&self) -> Result<()> {
		let mut txh_g = self.txn_holder.lock().await;
		if let Some(mut txn_holder) = txh_g.take() {
			// Take the TxnHolder out of the Option
			if txn_holder.counter > 1 {
				txn_holder.counter -= 1;
				let _ = txh_g.replace(txn_holder); // Put it back if not the last reference
			} else {
				// Perform the actual rollback
				txn_holder.txn.rollback().await?;
				// No need to replace, as we want to leave it as None
			}
			Ok(())
		} else {
			Err(Error::NoTxn)
		}
	}

	pub async fn commit_txn(&self) -> Result<()> {
		if !self.with_txn {
			return Err(Error::CannotCommitTxnWithTxnFalse);
		}

		let mut txh_g = self.txn_holder.lock().await;
		if let Some(txh) = txh_g.as_mut() {
			let counter = txh.dec();
			// If 0, then, it should be matching commit for the first first begin_txn
			// so we can commit.
			if counter == 0 {
				// here we take the txh out of the option
				if let Some(txn) = txh_g.take() {
					txn.txn.commit().await?;
				} // TODO: Might want to add a warning on the else.
			} // TODO: Might want to add a warning on the else.

			Ok(())
		}
		// Ohterwise, we have an error
		else {
			Err(Error::TxnCantCommitNoOpenTxn)
		}
	}

	pub fn db(&self) -> &Pool<Postgres> {
		&self.db_pool
	}

	pub async fn fetch_one<'q, O, A>(
		&self,
		query: QueryAs<'q, Postgres, O, A>,
	) -> Result<O>
	where
		O: for<'r> FromRow<'r, <Postgres as sqlx::Database>::Row> + Send + Unpin,
		A: IntoArguments<'q, Postgres> + 'q,
	{
		let data = if self.with_txn {
			let mut txh_g = self.txn_holder.lock().await;
			if let Some(txn) = txh_g.as_deref_mut() {
				query.fetch_one(txn.as_mut()).await?
			} else {
				query.fetch_one(self.db()).await?
			}
		} else {
			query.fetch_one(self.db()).await?
		};

		Ok(data)
	}

	pub async fn fetch_optional<'q, O, A>(
		&self,
		query: QueryAs<'q, Postgres, O, A>,
	) -> Result<Option<O>>
	where
		O: for<'r> FromRow<'r, <Postgres as sqlx::Database>::Row> + Send + Unpin,
		A: IntoArguments<'q, Postgres> + 'q,
	{
		let data = if self.with_txn {
			let mut txh_g = self.txn_holder.lock().await;
			if let Some(txn) = txh_g.as_deref_mut() {
				query.fetch_optional(txn.as_mut()).await?
			} else {
				query.fetch_optional(self.db()).await?
			}
		} else {
			query.fetch_optional(self.db()).await?
		};

		Ok(data)
	}

	pub async fn fetch_all<'q, O, A>(
		&self,
		query: QueryAs<'q, Postgres, O, A>,
	) -> Result<Vec<O>>
	where
		O: for<'r> FromRow<'r, <Postgres as sqlx::Database>::Row> + Send + Unpin,
		A: IntoArguments<'q, Postgres> + 'q,
	{
		let data = if self.with_txn {
			let mut txh_g = self.txn_holder.lock().await;
			if let Some(txn) = txh_g.as_deref_mut() {
				query.fetch_all(txn.as_mut()).await?
			} else {
				query.fetch_all(self.db()).await?
			}
		} else {
			query.fetch_all(self.db()).await?
		};

		Ok(data)
	}

	pub async fn execute<'q, A>(&self, query: Query<'q, Postgres, A>) -> Result<u64>
	where
		A: IntoArguments<'q, Postgres> + 'q,
	{
		let row_affected = if self.with_txn {
			let mut txh_g = self.txn_holder.lock().await;
			if let Some(txn) = txh_g.as_deref_mut() {
				query.execute(txn.as_mut()).await?.rows_affected()
			} else {
				query.execute(self.db()).await?.rows_affected()
			}
		} else {
			query.execute(self.db()).await?.rows_affected()
		};

		Ok(row_affected)
	}
}
