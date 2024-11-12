
## Reference

https://www.youtube.com/watch?v=3cA_mk4vdWY&list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q


## Summary of improvements

 - `mw_response_map` to also log responses
 - `mw_ctx_resolve` (auth-resolve) to validate token (jwt ?)
 - complete login/logoff
 - encryption ? tls ?
 - ModelManager
 - Write to DB

## Tip - Env vars

Save your env vars under `.cargo/config.toml` For eg

```toml
[env]
foo="bar"
```

read it like this

```rust
fn main() {
    let foo = env!("foo");
    println!("{}", foo);
}
```

 - Keeps env separate
 - Use it to contain your secrets etc in k8s settings he says! 

## Process for this video

I am not actually coding along but following his updated code base for rust-10x.

## Chapter 2 - Add tracing

> 🔗 [Chapter 2 Link](https://www.youtube.com/watch?v=3cA_mk4vdWY&list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q&index=21&t=590s)

### Add tracing dependencies

> They make a distinction between the words logging and tracing. What I used to think of as logging, log4j etc, they call tracing. Logging in their context is surfacing server errors to a log-store service on a cloud that is used for alerting ops etc folk.

```diff
[dependencies]
tracing = "0.1"
tracing-subscriber = {version = "0.3", features=["env-filter"]}
```

 - `env-filter` features allows for configuration of log-levels via env variables


### Add tracing

```rust
// setup
tracing_subscriber::fmt()
    .without_time()     // for early local development
    .with_target(false) // ??
    .with_env_filter(EnvFilter::from_default_env())
    .init();


// usr
info!("...");

debug!(""); 
```

## Chapter 3 - Add Configuration

> 🔗 [Chapter 3 Link](https://www.youtube.com/watch?v=3cA_mk4vdWY&list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q&index=21&t=866s)

Modifies the `.cargo/config.toml`

```diff
+ ## -- ConfigMap 
+ SERVICE_WEB_FOLDER="web-folder/"
```

 - Comments say that this should be used only for cargo commands
 - for deployed env (k8s), they should be managed by k8s
   - Meaning ? Don't pack the .cargo folder into the container ?
   - Have cargo build somehow skip it ?
 - Says that the convention is to start with `SERVICE_` and use snake case upper case for the rest.
 - 👉 paths will be relative to the location of the `Cargo.toml` file.

 
 Final code looks like below. Watch the video for the evolution of this design from 18min time stamp.

```rust
use lib_utils::envs::get_env;
use std::sync::OnceLock;

pub fn web_config() -> &'static WebConfig {
	static INSTANCE: OnceLock<WebConfig> = OnceLock::new();

	INSTANCE.get_or_init(|| {
		WebConfig::load_from_env().unwrap_or_else(|ex| {
			panic!("FATAL - WHILE LOADING CONF - Cause: {ex:?}")
		})
	})
}

#[allow(non_snake_case)]
pub struct WebConfig {
	pub WEB_FOLDER: String,
}

impl WebConfig {
	fn load_from_env() -> lib_utils::envs::Result<WebConfig> {
		Ok(WebConfig {
			WEB_FOLDER: get_env("SERVICE_WEB_FOLDER")?,
		})
	}
}
```

 - `WebConfig` struct
 - Static instance of it to avoid re-loading it each time via the new `OnceLock` mechanism
 - panic early so app crashes instead of proceeding without config
 - `env![]` or some such should be usable. Not sure what special things lib_utils::envs::get_env does.

```rust
pub enum Error {
	MissingEnv(&'static str),
	WrongFormat(&'static str),
}

pub fn get_env(name: &'static str) -> Result<String> {
	env::var(name).map_err(|_| Error::MissingEnv(name))
}
```

and this is used as 

```rust
use crate::web_config;

any_service(
    ServeDir::new(&web_config().WEB_FOLDER)...;
)
```

 - `&` to use a reference to the string


## Notes on testing

> [Test Philosophy link](https://youtu.be/3cA_mk4vdWY?list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q&t=1289)

Unit tests for layers with dependencies can be done in two ways
 - Mock impls of those dependencies (say DB etc)

![Testing notes](./img/testing-notes.svg)

## Chapter 4 - DB Live reload

> [Chapter 4 link](https://youtu.be/3cA_mk4vdWY?list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q&t=1483)
> Video shows monolithic project but for the final impl, this is in `lib-core/Cargo.toml`

```toml
[dependencies]
...
# -- Data
sqlx = { version = "0.7", features = [ "macros", "runtime-tokio", "postgres", "uuid" ] }
```

![DB development and live-reload](./img/init-db-live-reload.svg)

Actual sql files go under a `sql` folder and `sql\00-dev-initial`

![DB Initialization on startup](./img/init-db-init.svg)

## sql - drop
see `sql\dev_initial\00-recreate-db.sql`

```sql
-- DEV ONLY - Brute Force DROP DB (for local dev and unit test)
SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE
 usename = 'app_user' OR datname = 'app_db';
DROP DATABASE IF EXISTS app_db;
DROP USER IF EXISTS app_user;

-- DEV ONLY - Dev only password (for local dev and unit test).
CREATE USER app_user PASSWORD 'dev_only_pwd';
CREATE DATABASE app_db owner app_user ENCODING = 'UTF-8';
```

 - drops all current sessions
 - assumes `app_db` database and `app_user` user
 - run as `db_user`

## sql - create schema
see `sql\dev_initial\02-dev-seed.sql`

Too large to list here but straightforward stuff. Some things are new for me (_weak sql-fu circa 1999 on an Oracle db_).

```sql
-- seems to specify user_type as holding either `Sys` or `User`
CREATE TYPE user_typ AS ENUM ('Sys', 'User');
```

Tables

  - user (id, username, user_type, pwd, pwd_salt, token_salt and timestamps)
  - agent
  - conv
  - conv_user
  - conv_msg


## sql - seed db

see `sql\dev_initial\02-dev-seed.sql`

```sql
-- root user (at id = 0)
INSERT INTO "user" 
    (id,  typ, username, cid, ctime, mid, mtime) VALUES 
    (0, 'Sys', 'root',  0,   now(), 0,   now());

-- User demo1
INSERT INTO "user" 
    (username, cid, ctime, mid, mtime) VALUES 
    ('demo1',  0,   now(), 0,   now());

-- Agent mock-01 (with 'parrot' model) (id: 100)
INSERT INTO "agent"    
    (id,  owner_id, name,      cid, ctime, mid, mtime) VALUES
    (100, 0,        'mock-01', 0,   now(), 0,   now());
```

## _dev_utils module

> JC put this under `lib-core`. The model, db code, crud macros are all in this module.

The module introduces the use of toki's `OnceCell` which allows for static initialization of the db (_the `OnceLock` is for non-async inits while `OnceCell` is for async ones_)

```rust
/// Initialize environment for local development.
/// (for early development, will be called from main()).
pub async fn init_dev() {
	static INIT: OnceCell<()> = OnceCell::const_new();

	INIT.get_or_init(|| async {
		info!("{:<12} - init_dev_all()", "FOR-DEV-ONLY");

		dev_db::init_dev_db().await.unwrap();
	})
	.await;
}
```

## later changes

 Post this video, looks like a bunch more changes have happened.

  - _dev_utils::{`seed_users`, `seed_user`, `clean_users`, `


## Tying it all together

![Rust web-app deployment component diagram](./img/rustwebapp-deployment.svg)

## Chapter 5 - Model 1/2 Task CRUD

![Model and CRUD mindmap](./img/model-and-crud.svg)


### Creating DB pool

> code in `lib_core\src\_dev_utils\dev_db.rs`

```rust
// NOTE: Hardcode to prevent deployed system db update.
const PG_DEV_POSTGRES_URL: &str = "postgres://postgres:welcome@localhost/postgres";
const PG_DEV_APP_URL: &str = "postgres://app_user:dev_only_pwd@localhost/app_db";

pub asyc fn init_dev_db() {
    ..
    let app_db = new_db_pool(PG_DEV_APP_URL);
}

async fn new_db_pool(db_con_url: &str) -> Result<Db, sqlx::Error> {
	PgPoolOptions::new()
		.max_connections(1)
		.acquire_timeout(Duration::from_millis(500))
		.connect(db_con_url)
		.await
}
```

### Creating the ModelManager


```rust
// 👉 lib_core\src\_dev_utils\dev_db.rs
use tokio::sync::OnceCell;
..

pub async fn init_test() -> ModelManager {
	static INIT: OnceCell<ModelManager> = OnceCell::const_new();

	let mm = INIT
		.get_or_init(|| async {
			init_dev().await;
			// NOTE: Rare occasion where unwrap is kind of ok.
			ModelManager::new().await.unwrap()
		})
		.await;

	mm.clone()
}
```



```rust
// 👉 lib_core\src\model\mod.rs
#[cfg_attr(feature = "with-rpc", derive(rpc_router::RpcResource))]
#[derive(Clone)]
pub struct ModelManager {
	dbx: Dbx,
}

impl ModelManager {
	/// Constructor
	pub async fn new() -> Result<Self> {
		let db_pool = new_db_pool()
			.await
			.map_err(|ex| Error::CantCreateModelManagerProvider(ex.to_string()))?;
		let dbx = Dbx::new(db_pool, false)?;
		Ok(ModelManager { dbx })
	}

	pub fn new_with_txn(&self) -> Result<ModelManager> {
		let dbx = Dbx::new(self.dbx.db().clone(), true)?;
		Ok(ModelManager { dbx })
	}

	pub fn dbx(&self) -> &Dbx {
		&self.dbx
	}
}
```

### Create Model entity - Task

```rust
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
```

He is combining two roles here.
 - `Task` as something to serialize into the DB
 - `Task` as something to serialize into JSON for logging purposes ?
 - `Task slices` coming in from various CRUD calls. These are JSON Deserializable only, _why nor Serialize also for logging?_ . He says, treat **structs as views into the database table**. Things like generatedID, time-stamps etc belong in the db, private to the impl.
   - `TaskForCreate`
   - `TaskForUpdate`


### Create BMC for model entity - Task create

```rust
pub struct TaskBMC;

impl TaskBMC {
    pub async fn create(
        _ctx: &Ctx,             // common state
        mm: &ModelManager,      // common state
        task_c: TaskForCreate   // task/method specific payload
    ) -> Result<i64> {
        
        // get the pool
        let db = mm.db();

        let (id,) = sqlx::query_as::<_, (i64,)>(
            "INSERT INTO task (title) values ($1) returning id"
        )
        .bind(task_c.title)
        .fetch_one(db)
        .await?;

        Ok(id)
    }    
}

// since we use await?
// we need a way to convert sqlx errors into our model Errors.
// Theres an evolution from useing `serde_as` to `#[from]`
pub enum Error {
    ..
    Sqlx(sqlx::Error),
    ...
}

impl From<sqlx::Error> Error {
    pub fn from(value: sqlx::Error) -> Self {
        Self::Sqlx(value)
    }
}
```

 - Starting with CRUD 
 - Can be anything the entity needs
 - Ctx is de-coupled from front-end tech. Could be web, could be desktop etc. All that the model needs to know that it will get a Ctx to get info about the user etc.
 - 👉 eventually, put security at the Model layer. So that irrespective of how things some in, you only allow authenticated and authorized users.
 - 👉 `Ctx, ModelManager` will be present for all BMC calls.

### Testing BMC - task create

> Run tests
> - all       `cargo watch -q -c -x "test -- --nocapture"`
> - specified `cargo watch -q -c -x "test test_create_ok -- --nocapture"`
> - module    `cargo watch -q -c -x "test model::task::tests -- --nocapture"`
>
> cargo tests are run in paralell. Force them to be serial by using the `#[serial]` annotation from a new dev-dependency of `serial_test="3"`. This slows things down but is simple unless you have a separate serialization mechanism for your db access.

Note: When running, he uses the `--nocapture` because of print statements in the output. However, even without that, the test runner does print out the name of each test.

```rust
#[cfg(test)]
mod tests {
    #![allow(unused)]
    use super::*;
    use anyhow::Result;

    // practive to prefix with `test_` but not mandatory
    #[serial]
    #[tokio::test]
    async fn test_create_ok() -> Result<()> {
        // setup and fixture
        // see `lib_core::_dev_utils::init_test()`
        let mm  = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();
        let fx_title = "test_create_ok title"; // prefix with fx to indicate fixture.

        // -- test body
        let fx_task_c = TaskForCreate {
            title: fx_title.to_string(),
        };

        // 1 - create in bmc
        let id = TaskBmc::create(&ctx, &mm, fx_task_c).await?;

        // 2 - fetch from bmc for compare
        let (title,): (String,) = 
            sqlx::query_as("SELECT title from task where id = $1")
            .bind(id)
            .fetch_one(mm.db)
            .await?;

        // 3 - compare with expected
        assert_eq!(title, fx_title);

        // 4 - cleanup
        let count = sqlx::query("DELETE FROM task WHERE id = $1")
        .bind(id)
        .execute(mm.db)
        .await?
        .rows_affected();

        // print out a line for each test. Won't the test runner do this ??
        println!("->> {title}");
        assert_eq!(count, 1, "Did not delete 1 row in cleanup of test_create_ok!");

        Ok(())
    }
}
```

  - `#[cfg(test)]` comes in how ? `cargo run --test` 
  - `anyhow::Result` avoid stressing about error ?
  - `use super::*` use all of parent module's stuff
  - test positive cases
    - test_create_ok
  - test error cases
    - test_create_err_duplicate

### test fixtures

Mock the data that the web layer provides in it's state.
 - ModelManager _(`lib_core::model::ModelManager`)_
 - Ctx _(`lib_core::ctx::Ctx`)_

He has this additional method in the `lib_core::_dev_utils` module

```rust
/// Initialize test environment.
pub async fn init_test() -> ModelManager {
	static INIT: OnceCell<ModelManager> = OnceCell::const_new();

	let mm = INIT
		.get_or_init(|| async {
			init_dev().await;
			// NOTE: Rare occasion where unwrap is kind of ok.
			ModelManager::new().await.unwrap()
		})
		.await;

	mm.clone()
}
```

 - calls `init_dev()` and then `ModelManager::new()` which `main()` does.

### Create BMC for model entity - Task get

```rust
impl TaskBmc {
    ..
    pub async fn get(_ctx: &Ctx, mm &ModelManager, id:i64) -> Result<Task> {
        let db = mm.db();

        // .await? error path if problem with select query
        //.ok_or error if fetch_optional returns None.
        let task: Task = sqlx::query_as("SELECT * from task where id = $1")
            .bind(id)
            .fetch_optional(db)
            .await?
            .ok_or(Error::EntityNotFound{entity: "task", id})?;

        Ok(task);
    }
}
```

 - `.fetch_optional(db)` returns option for us to deal with later
 - `ok_or` allows su to report the right error message if option return is None
 - ❓ _Now how does this return the actual Task instance ? The db return will be the row. Note that Task is {id, string}, will DB return implicitly create an instance?_ 
   - ✔️ The **FromRow** derivation is where this happens. A sqlx artifact that allows creation of structs from a table row. _Likely has constraints of field names as column names etc_. See `#[derive(..,FromRow,..)]` on `Task`

### Create BMC for model entity - Task delete

```rust
impl TaskBmc {
    ..
    pub async fn delete(_ctx: &Ctx, mm &ModelManager, id:i64) -> Result<()> {
        let db = mm.db();

        // .await? error path if problem with select query
        //.ok_or error if fetch_optional returns None.
        let count = sqlx::query_as("DELETE from task where id = $1")
            .bind(id)
            .execute(db)
            .await?
            .rows_affected();

        if count == 0 {
            return Err(Error::EntityNotFound{entity: "task", id});
        }

        Ok(());
    }
}
```

straightforward.

### Cleanup tests - Task create

He then replaces the low-level sql portions of the test with the new `get` and `delete` methods. Caveat is that testing some functions using other functions is not always great but neither is duplicating code. Judgement I guess.

Replace

```rust
        // 2 - fetch from bmc for compare
        let (title,): (String,) = 
            sqlx::query_as("SELECT title from task where id = $1")
            .bind(id)
            .fetch_one(mm.db)
            .await?;

        // 3 - compare with expected
        assert_eq!(title, fx_title);

        // 4 - cleanup
        let count = sqlx::query("DELETE FROM task WHERE id = $1")
        .bind(id)
        .execute(mm.db)
        .await?
        .rows_affected();
```

with

```rust
        // 2 - fetch from bmc for compare
        let task = TaskBmc::get(&ctx, &mm, id).await?;
        assert_eq!(task.title, fx_title);

        // 3 - cleanup
        TaskBmc::delete(&ctx, &mm, id).await?;
```

### create Bmc test - test not found

```rust
#[serial]
    #[tokio::test]
    async fn test_get_err() -> Result<()> {
        // setup and fixture
        // see `lib_core::_dev_utils::init_test()`
        let mm  = _dev_utils::init_test().await;
        let ctx = Ctx::root_ctx();

        // Unknown id 
        let fx_id = 100; // Known no id: DB starts with 1000.            

        // 1 - fetch from bmc
        let res = TaskBmc::get(&ctx, &mm, fx_id).await;

        // 2 check
        // Note: Cannot use variables in the matches! macro apparently
        //       Hence the constants.
        assert!(
            matches!(
                res,
                Err(Error::EntityNotFound {
                    entity: "task",
                    id: 100
                })
            )
        )

        Ok(())
    }
```

You can add tests for delete etc also similarly.

```rust
pub async fn list(_ctx: &Ctx, mm: &ModelManager) -> Result<Vec<Task>> {
    let db = mm.db();
    let tasks: Vec<Task> = sqlx::query_as("SELECT * FROM task ORDER by id")
        .fetch_all(db)
        .await?;

    Ok(tasks)
}
```

also look at
  - `ssed*` methods

## Chapter 6 - Model 2/2 Shared Impl

 - There are common patterns for all entity types when it comes to fetching from db, single vs multiple rows, deletion etc.
 - So lots to gain from sharing common code
   - no inheritence so cant use that.
   - *traits* - Blanket and Default impl
   - *generics* - Monomorphisation & Dyn
   - *macros* - declarative and procedural macros
 - Files
   - `lib_core\src\model\base\*`


### Common Bmc

developing it seems pretty standard. Take the concrete impl and parameterize it by type. Great to be guided through the development process by J. Chone.

```rust
pub trait DbBmc {
    const TABLE: &'static str;
}

pub async fn get<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E> 
where 
    MC: DbBmc,
    E : for<'r> FromRow<'r, PgRow> + Unpin + Send
{
    todo!()
}
```

 - Note: Each BMC is for one entity type which has one TABLE
 - get is parameterized with
   - `MC` the ModelController, `DbBmc` for instance
   - `E` for the entity under consideration (`Task` say)
 - type constraints
   - `MC` needs to implement `DbBmc` trait
   - `E` need to implement `FromRow` _sqlx: allows to return a struct from a Row_. Start with `E : FromRow` and notice it won't compile.
     - `FromRow<'r, R>` takes a lifetime and a generic parameter.
     - introduces **higher type bounds**
     - 👉 `E : for<'r> FromRow<'r, PgRow>`. `PgRow` is what it is converting from, so this is ok. The _higher type bound_ of `for<'r>`, I will figure out later.
     - 👉 Because `E` is used in async, also needs `+ Unpin + Send` _and now I am getting into the super cryptic stuff that is the bread and butter of low level rust looks like_.

**Now copy the non-generic impl of task and tweak from there**

```rust
pub async fn get<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E> 
where 
    MC: DbBmc,
    E : for<'r> FromRow<'r, PgRow> + Unpin + Send
{
     let db = mm.db();

        // .await? error path if problem with select query
        //.ok_or error if fetch_optional returns None.
        let task: Task = sqlx::query_as("SELECT * from task where id = $1")
            .bind(id)
            .fetch_optional(db)
            .await?
            .ok_or(Error::EntityNotFound{entity: "task", id})?;

        Ok(task);
}
```

⏬

```rust
pub async fn get<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E> 
where 
    MC: DbBmc,
    E : for<'r> FromRow<'r, PgRow> + Unpin + Send
{
     let db = mm.db();

        // .await? error path if problem with select query
        //.ok_or error if fetch_optional returns None.
        let sql = format!("SELECT * from {} where id = $1", MC::TABLE);
        let entity: E = sqlx::query_as(&sql)
            .bind(id)
            .fetch_optional(db)
            .await?
            .ok_or(Error::EntityNotFound{entity: MC::TABLE, id})?;

        Ok(entity);
}
```

Convert over existing Bmc to the new DbBmc

```rust
pub struct TaskBmc;

impl DbBmc for TaskBmc {
    const TABLE: &'static str = "task";
}

implr TaskBmc {
    pub async fn get(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<Task> {
        // Pass TaskBmc as the DbBmc impl and Task as E
        // Since the compiler can infer based on return type, this can also be
        //   base::get<Self, _>(_ctx, mm, id).await
        base::get<Self, Task>(_ctx, mm, id).await
    }
}
```

He now starts talking about `update` and asks about what type the update payload should take. I would have thought simply another type but he continues to the need for a sql builder and introduces a custom library (_his own_) called `👉 sqlb`, a high-level builder pattern for sql quearies. _Now, this seems to either be a pedagogical tool or a temporary diversion but the final code uses `sea` and `modql` as dependencies_.

For eg..

```rust
let entity: E = sqlb::select()
    .table(MC::TABLE)
    .and_where("id", "=", id)
    .....
```

Then he says, this select does a `SELECT *` and we can/need-to more granular to put the view in there (subset of fields). To this end:
 
```rust 
use sqlb::{Fields, HasFields}

#[derive(Fields)]
pub struct Task {
    ..

    // skip the field below. Does not map to a column
    [#field(skip)]
    someFieldToSkip : someType,

    // Maps to a column but with a different name
    // Also update sqlx annotation so it can do the FromRow conversion to this type
    [#field(name="useThisColumnName")]
    [#sqlx(rename="useThisColumnName")]
    someRandomName : someType,
}

pub async fn get<MC, E>(_ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<E> 
where 
    MC: DbBmc,
    E : for<'r> FromRow<'r, PgRow> + Unpin + Send
    E : HasFields,
{
     let db = mm.db();

        // .await? error path if problem with select query
        //.ok_or error if fetch_optional returns None.
        let sql = format!("SELECT * from {} where id = $1", MC::TABLE);
        let entity: E = sqlb::select()
            .table(MC.TABLE)
            .columns(E::field_names)
            .and_where("id", "=", id)
            .fetch_optional(db)
            .await?
            .ok_or(Error::EntityNotFound{entity: MC::TABLE, id})?;

        Ok(entity);
}
```


Now implement `list` which seems to be trivial the way it is (_no filter_) but adds a `order_by`.

```rust
pub async fn list<MC, E>(_ctx: &Ctx, mm: &ModelManager) -> Result<Vec<E>>
where 
    MC: DbBmc,
    E : for<'r> FromRow<'r, PgRow> + Unpin + Send
    E : HasFields,
{
     let db = mm.db();

        // .await? error path if problem with select query
        //.ok_or error if fetch_optional returns None.
        let sql = format!("SELECT * from {} where id = $1", MC::TABLE);
        let entities: Vec<E> = sqlb::select()
            .table(MC.TABLE)
            .columns(E::field_names)
            .order_by("id")
            .fetch_all(db)
            .await?;

        Ok(entitie);
}
```

Some other developments

 - `create` which uses `fields = E::not_none_fields` and `sqlb::insert().data(fields)..returning(&["id"])` which gets the fields that are non-null (regular fields as well as Options) and the returns needed.
 - `update` which uses `fields = E::not_none_fields` and `sqlb::update().and_where("id", "=", id).data(fields).exec()`
 - `delete` is simpler and similar with `sqlb::delete().and_where(..)`


## Chapter 7 - Password encryption and validation

Ref: https://www.youtube.com/watch?v=3cA_mk4vdWY&list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q&index=21&t=6240s

![Password encrption and validation](./img/auth-pwd-encryption-validation.svg)

### User entity

see `lib_core\src\model\user.rc`

Note:
  - Have a simple `User` struct without any password (hashed) etc.
  - Have a more full `UserForLogin` meant for auth only. How would you restrict this though: _via visibility if everything is in the module I guess_ or _an API method that takes in the needed values and then does it_. Whats the visibility goal here ?


here, he is putting the needed sql derivations in a base trait like to

```rust
// Marker trait
pub trait UserBy: HsFields, + for<'r> FromRow<'r, PgRow> + Unpin + Send {}

impl UserBy for User         {}
impl UserBy for UserForLogin {}
impl UserBy for UserForAuth  {}
```

This then allows a generic `get` method that depends on `UserBy`. Something we'd typically do in a OO setting as well.

```rust
impl UserBmc {
    pub async fn get<E>(ctx: &Ctx, mm: &ModelManager, id:i64) -> Result<E> 
    where
        E: UserBy,
    {
        base::get<Self, _>(ctx, mm, id).await
    }
}
```

### update schema

`01-create-scherma.sql`

```diff
CREATE TABLE "user" (
    ..
+   pwd varchar(256),
+   pwd_salt uuid NOT NULL DEFAULT gen_random_uuid(),
+   token_salt uuid NOT NULL DEFAULT gen_random_uuid(),
...
)
```

## add deps for encryption

```toml
rand = "0.8"
hmac = "0.12"
sha2 = "0.10"
base64 = "2"
```

## encryption

Says encryption but seems to be using sha digest stuff. Would people call this encryption ? For one way, maybe digest is all that is needed. Two way maybe call it encryption.

I can get to this later. for now, just listening without much note-taking.

## key

HMAC takes a key. For the app, the has vreated a keygen exe which simply prints out a base64 encoded random key and plans to store it in the config file (or approp secrets). This is under `tools/genkey`

```rust
pub type Result<T> = core::result::Result<T, Error>;
pub type Error = Box<dyn std::error::Error>; // Ok for tools.

use lib_utils::b64::b64u_encode;
use rand::RngCore;

fn main() -> Result<()> {
	let mut key = [0u8; 64]; // 512 bits = 64 bytes
	rand::thread_rng().fill_bytes(&mut key);
	println!("\nGenerated key from rand::thread_rng():\n{key:?}");

	let b64u = b64u_encode(key);
	println!("\nKey b64u encoded:\n{b64u}");

	Ok(())
}
```

He creates two keys to save

 - SERVICE_PWD_KEY  _uses `get_env_b64u_as_u8s`_
 - SERVICE_TOKEN_KEY _uses `get_env_b64u_as_u8s`_
 - SERVICE_TOKEN_DURATION_SEC=1800 _Always suffix with time units so you don't confuse it as millis etc_. _uses `get_env_parse`_

These are defined as follows `use lib-utils::envs::*`

```rust
use crate::b64::b64u_decode;
use std::env;
use std::str::FromStr;

pub fn get_env(name: &'static str) -> Result<String> {
	env::var(name).map_err(|_| Error::MissingEnv(name))
}

pub fn get_env_parse<T: FromStr>(name: &'static str) -> Result<T> {
	let val = get_env(name)?;
	val.parse::<T>().map_err(|_| Error::WrongFormat(name))
}

pub fn get_env_b64u_as_u8s(name: &'static str) -> Result<Vec<u8>> {
	b64u_decode(&get_env(name)?).map_err(|_| Error::WrongFormat(name))
}
```

 - `T: FromStr` is part of the rust std lib

### Updating web-frontend to use hashed pwds

```rust
pub fn routes() -> Router {
    Router::new().route("/api/login", post(api_login_handler))
}

async fn api_login_handler(
    cookies: Cookies,
    payload: Json<LoginPayload>,
) -> Result<Json<Value>> {

}

#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: String,
    pwd : String,
}
```

**Start using ModelManager** as axum state.

```rust
// He just used mm: ModelManager. no & ??
// k. He's sending it in via mm.clone()
pub fn routes(mm: &ModelManager) -> Router {
    Router::new().route("/api/login", post(api_login_handler))
    .with_state(mm)
}

// State from axum::extract::State
async fn api_login_handler(
    State(mm) : State<ModelManager>,
    cookies: Cookies,
    payload: Json<LoginPayload>,
) -> Result<Json<Value>> {

    // note. When destructuring, field name should match. 
    // this allows `pwd_clear` in payload to be used for `pwd`
    let LoginPay {
        username,
        pwd: pwd_clear,
    } = payload;    

    // Note: Ctx is simply user context (user_id, conv_id)
    let root_ctx = Ctx::root_ctx();

    // -- get the user
    // Says: Never log this. Sometimes users type in pwd for username
    // and we don't want those showing up in the logs.
    let user: UserforLogin = UserBmc::first_by_username(&root_ctx, &mm, &username)
        .await?
        .ok_or(Error::LoginFailUsernameNotFound)?;

    let user_id = user.id;
}
```

Standard fix for **converting Bmc/model error to web-layer** to allow the `await?` to work.

```diff
// in web/error.rs
pub enum Error {
+   LoginFailUsernameNotFound,    
    ..
+    // Model errors
+   Model(model::Error),

}

// froms
+impl From<model::Error> for Error {
+    fm from(val: model::Error) -> Self {
+        Self::Model(val)
+    }
+}

```

**Validate the password**

```rust
// checks output of bmc/internal-db
// some of these usages would not be obvious to me. I would start with if/else
// and then assign to Some(pwd) or have a small method and call it with a `?`;
let Some(pwd) = user.pwd else {
    return Err(Error::LoginFailUserHasNoPwd{user_id});
}

// validate
pwd::validate_pwd(
    &EncryptContent {
        salt: user.pwd_salt.to_string(),
        content: pwd_clear.clone(),
    },
    &pwd, // hashed pwd
)
.map_err(|_| Error::LoginFailPwdNotMatching{user_id})?;

```


```diff
// web/error.rs
pub enum Error {
   LoginFailUsernameNotFound,    
+  LoginFailUserHasNoPwd {user_id:i64},
+  LoginFailPwdNotMatching {user_id:i64}
}
```

 - Note: We are logging user_id that comes from our db. Not user input so ok to log.
 - ❓what is `&EncryptContent` ? Whats the problem with simply `EncryptContent` and mem transfer semantics ?
 - `pwd::validate` returns a `crypt::Error`. Judgement call here to map it into a `web::Error` instead of including a `CryptError(crypt::Error)` and a `From`.

### Add few more details to client visible error

Note that we always map server errors to client errors explicitly in order to give as little information as possible (_to reduce revealing info that a malicious actor can use to attack_).

Normally, all new server errors will simply be service-errors as per

```rust
impl Error {
	pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
		use web::Error::*; // TODO: should change to `use web::Error as E`

        match self {
            ...
            // -- Fallback.
			_ => (
				StatusCode::INTERNAL_SERVER_ERROR,
				ClientError::SERVICE_ERROR,
			),
        }
    }
}
```

Now explicitly allow login errors through but again with just enough information

```diff
pub enum ClientError {
    ..
+	LOGIN_FAIL,
    ...
}

impl Error {
	pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
		use web::Error::*; // TODO: should change to `use web::Error as E`

        match self {
            ...
+           // -- Login
+			LoginFailUsernameNotFound
+			| LoginFailUserHasNoPwd { .. }
+			| LoginFailPwdNotMatching { .. } => {
+				(StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL)
+			}
            ...
            // -- Fallback.
			_ => (
				StatusCode::INTERNAL_SERVER_ERROR,
				ClientError::SERVICE_ERROR,
			),
        }
    }
}
```

## Chapter 8 - Secure token and Web Auth

Timestamped section: https://www.youtube.com/watch?v=3cA_mk4vdWY&list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q&index=21&t=8541s

 - http-only cookie : _scripts wont have access to it ?_
 - token has a signature to ensure tampering can be detected
 - downstream validation of token is handled by the `mw_ctx_resolve` layer

![Secure token mind map](./img/req-auth-token-mm.svg)

 - bunch of methods added to lib_utils (now_utc, format_time, now_utc_plus_sec_str, parse_utc))

### create the token

> https://www.youtube.com/watch?v=3cA_mk4vdWY&list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q
> 
> Why for token you decided to implement from scratch as JWT has pretty much the same structure, goal and it's already an industry standard?
> 
> @JeremyChone 7 months ago (edited) Those are good questions. I wanted to write the longer answer, but here is a relatively short one:
> 
> Secure Web Token vs. JWT: In our case, we assume the token is designed for interpretation solely by our  system. Different services may exist, but they all belong to the same organization. Hence, we don't need to  disclose the algorithm used, as sharing less is preferable. The design also aims to decouple Authorization  from Authentication. The token is used to verify identity, but roles, privileges, and other aspects are better managed by an independent API and re-evaluated at the time of request. Roles/Privileges should be  transactional, with tokens remaining valid longer than the period suitable for checking roles/privileges.  Therefore, roles in JWT are more cosmetic than security-based.
> In distributed systems where we don't own all the systems, JWT might be acceptable, though many don't share keys. However, one shouldn't fully trust the roles defined in JWT.
> 
> At its simplest and most secure, a token comprises three essential parts:
> 
> The identifier (e.g., username or user UUID),
> The expiration time,
> The signature, created with minimal information sharing.
> In summary, while JWT may be useful in a distributed context, in a multi-service system we own, adopting a minimalistic approach with less information sharing reduces the attack surface and enforces transactional  access control.
>
> 👉 code at `/lib-auth/src/token/mod.rs`


**Token struct and generation**

```rust
// ident_b64u.exp_b64u.sign_64u
pub struct Token {
    pub ident: String;
    pub expt : String,
    pub sign_b64u: String,
}

fn _generate_token(
	ident: &str,
	duration_sec: f64,
	salt: Uuid,
	key: &[u8],
) -> Result<Token> {
	// -- Compute the two first components.
	let ident = ident.to_string();
	let exp = now_utc_plus_sec_str(duration_sec);

	// -- Sign the two first components.
	let sign_b64u = _token_sign_into_b64u(&ident, &exp, salt, key)?;

	Ok(Token {
		ident,
		exp,
		sign_b64u,
	})
}
```

**Validate the token**: Signature validation against tampering as well as expiry.

```rust
fn _validate_token_sign_and_exp(
	origin_token: &Token,
	salt: Uuid,
	key: &[u8],
) -> Result<()> {
	// -- Validate signature.
	let new_sign_b64u =
		_token_sign_into_b64u(&origin_token.ident, &origin_token.exp, salt, key)?;

	if new_sign_b64u != origin_token.sign_b64u {
		return Err(Error::SignatureNotMatching);
	}

	// -- Validate expiration.
	let origin_exp = parse_utc(&origin_token.exp).map_err(|_| Error::ExpNotIso)?;
	let now = now_utc();

	if origin_exp < now {
		return Err(Error::Expired);
	}

	Ok(())
}
```

**Sign the token**
```rust
/// Create token signature from token parts
/// and salt.
fn _token_sign_into_b64u(
	ident: &str,
	exp: &str,
	salt: Uuid,
	key: &[u8],
) -> Result<String> {
	let content = format!("{}.{}", b64u_encode(ident), b64u_encode(exp));

	// -- Create a HMAC-SHA-512 from key.
	let mut hmac_sha512 = Hmac::<Sha512>::new_from_slice(key)
		.map_err(|_| Error::HmacFailNewFromSlice)?;

	// -- Add content.
	hmac_sha512.update(content.as_bytes());
	hmac_sha512.update(salt.as_bytes());

	// -- Finalize and b64u encode.
	let hmac_result = hmac_sha512.finalize();
	let result_bytes = hmac_result.into_bytes();
	let result = b64u_encode(result_bytes);

	Ok(result)
}
```

### Set/Clear token cookies

```rust
pub mod mw_req_stamp;

pub use self::error::{Error, Result};
use lib_auth::token::generate_web_token;
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;

// endregion: --- Modules

pub const AUTH_TOKEN: &str = "auth-token";

fn set_token_cookie(cookies: &Cookies, user: &str, salt: Uuid) -> Result<()> {
	let token = generate_web_token(user, salt)?;

	let mut cookie = Cookie::new(AUTH_TOKEN, token.to_string());
	cookie.set_http_only(true);
	cookie.set_path("/");

	cookies.add(cookie);

	Ok(())
}

fn remove_token_cookie(cookies: &Cookies) -> Result<()> {
	let mut cookie = Cookie::from(AUTH_TOKEN);
	cookie.set_path("/");

	cookies.remove(cookie);

	Ok(())
}
```

> Note: There is also a `set-secure` which forces the cookie to be returned only via https

 - **http-only** will prevent js/scripts from accessing the cookie
 - `set_path("/")`. 
   - The cookie will be accessible only to the URI set.
   - If no URI specified, it is the URI of the current request
   - If `/` is specified, it is special, means `global cookie`, send to every URI   

**Update login handler**

```diff
async fn api_login_handler(
	State(mm): State<ModelManager>,
	cookies: Cookies,
	Json(payload): Json<LoginPayload>,
) -> Result<Json<Value>> {
	debug!("{:<12} - api_login_handler", "HANDLER");

	let LoginPayload {
		username,
		pwd: pwd_clear,
	} = payload;
	let root_ctx = Ctx::root_ctx();

	// -- Get the user.
	let user: UserForLogin = UserBmc::first_by_username(&root_ctx, &mm, &username)
		.await?
		.ok_or(Error::LoginFailUsernameNotFound)?;
	let user_id = user.id;

	// -- Validate the password.
	let Some(pwd) = user.pwd else {
		return Err(Error::LoginFailUserHasNoPwd { user_id });
	};

	let scheme_status = pwd::validate_pwd(
		ContentToHash {
			salt: user.pwd_salt,
			content: pwd_clear.clone(),
		},
		pwd,
	)
	.await
	.map_err(|_| Error::LoginFailPwdNotMatching { user_id })?;

	// -- Update password scheme if needed
	if let SchemeStatus::Outdated = scheme_status {
		debug!("pwd encrypt scheme outdated, upgrading.");
		UserBmc::update_pwd(&root_ctx, &mm, user.id, &pwd_clear).await?;
	}

+	// -- Set web token.
+	web::set_token_cookie(&cookies, &user.username, user.token_salt)?;

	// Create the success body.
	let body = Json(json!({
		"result": {
			"success": true
		}
	}));

	Ok(body)
}
```

### Auth Resolve for requests - Full flow

Once the token is set, we need a way to validate and authorize it for each request. The two middlewares, `mw_ctx_require` and `mw_ctx_resolve` take care of this. The code below shows how the layers are setup. The `request` flows through the layers to the routes. The `response` of the route's handler then flows back out in reverse order. There are opportunities to add to the Request on the way in and to modify the response on the way out.


```rust
let routes_rpc = web::routes_rpc::routes(mm.clone())
    .route_layer(middleware::from_fn(mw_ctx_require));

let routes_all = Router::new()
		.merge(routes_login::routes(mm.clone()))
		.nest("/api", routes_rpc)
		.layer(middleware::map_response(mw_reponse_map))
		.layer(middleware::from_fn_with_state(mm.clone(), mw_ctx_resolver))
		.layer(CookieManagerLayer::new())
		.layer(middleware::from_fn(mw_req_stamp_resolver))
		.fallback_service(routes_static::serve_dir());
```

👉 Note that `Layers` are actually layered on top. So bottom-most layer is first in the chain. See the component diagram below for the sequence of layering chaining. Code extracts are listed under the diagram

![Request Auth full flow](./img/req-auth-full-flow-swimlanes.svg)

### layer 1 - mw_req_stamp_resolver

> Names are subjective. Since this is adding a request time-stamp, `resolver` seems less faithful to it's function than `mw_req_stamp_inserter`
>
> **Functionally**: Inserts a `uuid` and `time-stamp` into the Request's extensions store.
>
> **Axum Infra**: Provides an extractor so any downstream layer or handler can use `ReqStamp` or `Option<ReqStamp>` as parameter.
>
> **See** `web-server\src\web\mw_req_stamp.rs`

```rust
#[derive(Debug, Clone)]
pub struct ReqStamp {
	pub uuid: Uuid,
	pub time_in: OffsetDateTime,
}

pub async fn mw_req_stamp_resolver(
	mut req: Request<Body>,
	next: Next,
) -> Result<Response> {
	debug!("{:<12} - mw_req_stamp_resolver", "MIDDLEWARE");

	let time_in = now_utc();
	let uuid = Uuid::new_v4();

	req.extensions_mut().insert(ReqStamp { uuid, time_in });

	Ok(next.run(req).await)
}

// region:    --- ReqStamp Extractor
#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for ReqStamp {
	type Rejection = Error;

	async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
		debug!("{:<12} - ReqStamp", "EXTRACTOR");

		parts
			.extensions
			.get::<ReqStamp>()
			.cloned()
			.ok_or(Error::ReqStampNotInReqExt)
	}
}
```

### layer 3 - mw_ctx_resolver

> This does some heavy lifting and performs
> - validation of the token
> - setting a new token
> - create new Ctx instance (per-request Ctx)
>
> Note that this stores a `Result<CtxW>` so even if it fails, the layer passes the Request through. Any check for validity is done in the `mw_ctx_require` middle-ware (_which simply checks to see if the `Result<CtxW>` has a `Some` instead of an error_)


```rust
// IMPORTANT: This resolver must never fail, but rather capture the potential Auth error and put in in the
//            request extension as CtxExtResult.
//            This way it won't prevent downstream middleware to be executed, and will still capture the error
//            for the appropriate middleware (.e.g., mw_ctx_require which forces successful auth) or handler
//            to get the appropriate information.
pub async fn mw_ctx_resolver(
	State(mm): State<ModelManager>,
	cookies: Cookies,
	mut req: Request<Body>,
	next: Next,
) -> Response {
	debug!("{:<12} - mw_ctx_resolve", "MIDDLEWARE");

	let ctx_ext_result = ctx_resolve(mm, &cookies).await;

	if ctx_ext_result.is_err()
		&& !matches!(ctx_ext_result, Err(CtxExtError::TokenNotInCookie))
	{
		cookies.remove(Cookie::from(AUTH_TOKEN))
	}

	// Store the ctx_ext_result in the request extension
	// (for Ctx extractor).
	req.extensions_mut().insert(ctx_ext_result);

	next.run(req).await
}

async fn ctx_resolve(mm: ModelManager, cookies: &Cookies) -> CtxExtResult {
	// -- Get Token String
	let token = cookies
		.get(AUTH_TOKEN)
		.map(|c| c.value().to_string())
		.ok_or(CtxExtError::TokenNotInCookie)?;

	// -- Parse Token
	let token: Token = token.parse().map_err(|_| CtxExtError::TokenWrongFormat)?;

	// -- Get UserForAuth
	let user: UserForAuth =
		UserBmc::first_by_username(&Ctx::root_ctx(), &mm, &token.ident)
			.await
			.map_err(|ex| CtxExtError::ModelAccessError(ex.to_string()))?
			.ok_or(CtxExtError::UserNotFound)?;

	// -- Validate Token
	validate_web_token(&token, user.token_salt)
		.map_err(|_| CtxExtError::FailValidate)?;

	// -- Update Token
	set_token_cookie(cookies, &user.username, user.token_salt)
		.map_err(|_| CtxExtError::CannotSetTokenCookie)?;

	// -- Create CtxExtResult
	Ctx::new(user.id)
		.map(CtxW)
		.map_err(|ex| CtxExtError::CtxCreateFail(ex.to_string()))
}

// region:    --- Ctx Extractor
#[derive(Debug, Clone)]
pub struct CtxW(pub Ctx);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for CtxW {
	type Rejection = Error;

	async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
		debug!("{:<12} - Ctx", "EXTRACTOR");

		parts
			.extensions
			.get::<CtxExtResult>()
			.ok_or(Error::CtxExt(CtxExtError::CtxNotInRequestExt))?
			.clone()
			.map_err(Error::CtxExt)
	}
}
// endregion: --- Ctx Extractor

// region:    --- Ctx Extractor Result/Error
type CtxExtResult = core::result::Result<CtxW, CtxExtError>;

#[derive(Clone, Serialize, Debug)]
pub enum CtxExtError {
	TokenNotInCookie,
	TokenWrongFormat,

	UserNotFound,
	ModelAccessError(String),
	FailValidate,
	CannotSetTokenCookie,

	CtxNotInRequestExt,
	CtxCreateFail(String),
}
// endregion: --- Ctx Extractor Result/Error
```

### layer - mw_ctx_require

> All routes that need to enforce authentication should have this layer on them. Currently, the `api/rpc` routes do.

```rust
pub async fn mw_ctx_require(
	ctx: Result<CtxW>,
	req: Request<Body>,
	next: Next,
) -> Result<Response> {
	debug!("{:<12} - mw_ctx_require - {ctx:?}", "MIDDLEWARE");

	ctx?;

	Ok(next.run(req).await)
}
```


## Chapter 9 - RPC (JSON-RPC)

browser protocols

 - REST
 - GraphQL
 - gRPC
 - JSON RPC (_REST + RPC_)


### REST

Typical for CRUD operations on an entity, say `task`.

 - all methods are over HTTP 
 - POST `/api/tasks` to create. Send a JSON payload and receive a JSON payload with atleast the `id`.
 - GET  `/api/tasks/<id>` to get an entity given ID
 - GET  `/api/tasks?filter=..` to get multiple entities that match the query parameter.
 - PUT or PATCH  `/api/tasks/<id>` to update the entity. Send a JSON payload either partial or full based on your impl.
 - DELETE `/api/tasks/<id>` to delete the entity with supplied ID.

### JSON RPC

A lot like REST, except

 - all methods are POST
 - all methods target a single URI on the server. say `/api/rpc`
 - needs 3 fields
   - `id` which is used to allow the server to batch he says. Not clear. Can be null or empty. **Only contract is that the server send it back**
   - `method` which contains the actual method name. Server is expected to route to appropriate end-point.
   - `params` which is the payload of the object and is a JSON object.

 ```json
 // Payload will look like this
 {
    id: "rpc_req_id", //str, num or null
    method: "create task",
    params: {
        ...
    }
 }

// sucess response
// note that the id is reflected back
{
    id: "rpc_req_id",
    result : {
        ...
    }
}

// failure response
{
    id: "rpc_req_id",
    error: {
        code: 4000, //int
        message: "ERROR_MSG",
        data: ...., // optional
    }
}
```

## code structure

 - new module `wed/rpc/mod.rs`
 

### json rpc envelope

> Note that future refactoring will move this into his `rpc_router` lib as `rpc_router::request::Request`

A struct to hold the jrpc info, field names simply follow the spec. 
```rust
#[derive(Deserialize)]
struct RpcRequest {
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}
```

### json rpc crud helpers

The design thinking behind it is to generate wrapper types needed for wrapping the CRUD payloads. The idea is that client usages wrap their CRUD payloads in these structs and the intended macro expansions will route their payloads (_extracted from the wrappers_) correctly. These are under `lib-rpc-core\src\rpcs\rpc_params.rs`.

```rust
/// Params structure for any RPC Create call.
#[derive(Deserialize)]
pub struct ParamsForCreate<D> {
	pub data: D,
}

/// Params structure for any RPC Update call.
#[derive(Deserialize)]
pub struct ParamsForUpdate<D> {
	pub id: i64,
	pub data: D,
}
```

and `ParamsForCreate` is used in this fashion

```rust
pub async fn create_task (
    ctx : Ctx,
    mm: ModelManager,
    params: ParamsForCreate<TaskForCreate>,
) -> Result<Task> {    
    let id = TaskBmc::create(&ctx, &mm, params.data).await?;
    let task = TaskBmc::get(&ctx, &mm, id).await?;    
    Ok(task)
}


pub async fn list_tasks (
    ctx : Ctx,
    mm: ModelManager    
) -> Result<Task> {    
    ....
    Ok(tasks)
}

pub async fn update_task (
    ctx : Ctx,
    mm: ModelManager,
    params: ParamsForUpdate<TaskForUpdate>,
) -> Result<Task> {    
    ....
    Ok(task)
}
```

### axum routing of json rpc requests

```rust

pub fn routes(mm:ModelManager) -> Router {
    // Note that this will be nested under "/api" when
    // nested under the main router.
    // - we want the ModelManager as state for all the handlers under this.
    Router::new()
        .route("/rpc", post(rpc_handler))
        .with_state(mm)
}

async fn rpc_handler(
    State(mm): State<ModelManager>,
    ctx: Ctx,
    Json(rpc_req): Json<RpcRequest>,
) -> Response {
    // Why the redirect ?
    _rpc_handler(ctx, mm, rpc_req).await?.to_response()
}

async fn _rpc_handler(
    mm: ModelManager,
    ctx: Ctx,
    rpc_req: RpcRequest,
) -> Result<Json<Value>> {
    // destructure
    let RpcRequest {
        id: rpc_id,
        method: rpc_method,
        params: rpc_params
    } = rpc_req;

    debug!("..")

    // route by method name
    let result_json = match rpc_method.as_str() {
        "create_task" => {
            // params will be a Json value after this
            let params = rpc_params.ok_or(Error::RpcMissingParams{
                rpc_method: "create_task".to_string(),
            })?;

            // idiomatic rust uses variable shadowing so often!
            // better than trying to invent ridiculous names or 
            // nested scopes. But still a bit jarring after 
            // trying hard to avoid this in C++.
            let params = from_value(params).map_err(|_| Error::RpcFailJsonParams{
                rpc_method: "create_task".to_string(),
            })?;

            create_task(ctx, mm, params).await.map(to_value)??
        },
        "list_task" => list_tasks(ctx, mm).await.map(to_value)??,
        "update_task" => todo!(),
        "delete_task" => todo!(),

        // fallback
        _ => return Err(Error::RpcMethodUnknown(rpc_method)),
    };

    // json! macro comes from serde_json
    let body_response = json!({
        "id" : rpc_id,
        "result" : result_json
    })

    // Json here is an AXUM JSon Extractor/Response
    Ok(Json(body_response))
}
```

 - When mapping the results to json. As long as the result payload is decorated with `#[derive(Serialize)]`, simply use `some_func().await.map(|r| to_value(r));` to handle the mapping. This can be simplified to `some_func().await.map(to_value);`.
 - **list_tasks** is simple as it takes no params
   - the trailing `??` are cascades to perform unwrapping of the nested `Result<>` types.
 - **create_task** demonstrates flow with params
   - Checks for `Some` with an `ok_or`
   - Converts from JSon to the underlying struct instance via the `#[derive(Deserialize)]` annotation
   - ❓ dont understand the `"create_task".to_string()` yet.

### tying it all together: end to end 

```diff
+ let routes_rpc = rpc::routes(mm.clone)
+    .route_layer(middleware::from_fn(mw_ctx_require));

let routes_all = Router::new()
    .merge(routes_login::routes(mm.clone()))
+   .nest("/api", routes_rpc)
    .layer.....
```

test it qith quick_dev like this. **I still don't grok the usage of the `id` field**.

```rust
let req_list_tasks = hc.do_post(
    "/api/rpc",
    json!({
        "id" : 1,
        "method": "list_tasks"
    }),
    );
```

There is some obviosu repitition here. Repitiion for each method and repitition for each entity which needs CRUD support. He wants to solve this using declrative macros (_is there an other kind ?_)

> 👉 Makes the concrete code work (_just like I'd do in C++ templates_) and then convert to macro.

### macrofy list_tasks

Goal is to macrofy `list_tasks(ctx, mm).await.map(to_value)??`

**Start with basic macro shell**
```rust
macro_rules! exec_rpc_fn{
    // rpc call without params
    ($rpc_fn: expr, $ctx:expr, $mm:expr) => {
        todo!()
    }
}

...
list_tasks(ctx, mm).await.map(to_value)??
...
```

↓

**flesh out based on concrete function call**
```rust
macro_rules! exec_rpc_fn{
    // rpc call without params
    ($rpc_fn: expr, $ctx:expr, $mm:expr) => {
        $rpc_fn($ctx, $mm).await.map(to_value)??
    }
}

...
list_tasks(ctx, mm).await.map(to_value)??
...
```

↓

**replace concrete function call with macro incovation**
```rust
...
exec_rpc_fn!(list_tasks, ctx, mm)
...
```

### macrofy create_task

Goal is to macrofy the below

```rust
// params will be a Json value after this
let params = rpc_params.ok_or(Error::RpcMissingParams{
    rpc_method: "create_task".to_string(),
})?;

// idiomatic rust uses variable shadowing so often!
// better than trying to invent ridiculous names or 
// nested scopes. But still a bit jarring after 
// trying hard to avoid this in C++.
let params = from_value(params).map_err(|_| Error::RpcFailJsonParams{
    rpc_method: "create_task".to_string(),
})?;

create_task(ctx, mm, params).await.map(to_value)??
```

**Start with basic macro shell**
```rust
macro_rules! exec_rpc_fn{
    // rpc call with params
    ($rpc_fn: expr, $ctx:expr, $mm:expr, $json_params:expr) => {
        todo!()
    }
    ...
}
```

↓

**flesh out based on concrete function call**
```rust
macro_rules! exec_rpc_fn{
    // rpc call with params
    // Would have to send $fn_name separately but you can compute it
    ($rpc_fn: expr, $fn_name:expr, $ctx:expr, $mm:expr, $rpc_params:expr) => {
        {
            let rpc_fn_name = stringify!($rpc_fn);

            let params = $rpc_params.ok_or(Error::RpcMissingParams{
                rpc_method: rpc_fn_name.to_string(),
            })?;
            
            let params = from_value(params).map_err(|_| Error::RpcFailJsonParams{
                rpc_method: rpc_fn_name.to_string(),
            })?;

            $rpc_fn($ctx, $mm, params).await.map(to_value)??
        }
    }
    ...
}
```

↓

**replace concrete function call with macro incovation** and can do it for all the other methods as well
```rust
...
"create_task" => exec_rpc_fn!(create_task, ctx, mm, rpc_params),
"list_tasks"  -> exec_rpc_fn!(list_tasks , ctx, mm),
"update_task" => exec_rpc_fn!(update_task, ctx, mm, rpc_params),
"delete_task" => exec_rpc_fn!(delete_task, ctx, mm, rpc_params),
...
```

### adjust error info to propagate some more RPC error details

Since we adopt a security posture to provide as little information to the client as possible, a few things need to happen to propagate a new error to the user.

**Note** `web-server\web\error.rs` already has a `ModelError` defined for the model layer, like so

```rust
#[serde_as]
#[derive(Debug, Serialize, From, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
	...

	// -- Modules
	#[from]
	Model(model::Error),

    ...
}
```

**Add a new client error** in `web\error.rs` 

```diff
#[derive(Debug, strum_macros::AsRefStr)]
#[allow(non_camel_case_types)]
pub enum ClientError {
	LOGIN_FAIL,
	NO_AUTH,
+   ENTITY_NOT_FOUND { entity: &'static str, id: i64 },
    ...
}
```

**Update client error map** in `web\error.rs`
```diff
impl Error {
	pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
		use web::Error::*; // TODO: should change to `use web::Error as E`

		match self {
			// -- Login
			LoginFailUsernameNotFound
			| LoginFailUserHasNoPwd { .. }
			| LoginFailPwdNotMatching { .. } => {
				(StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL)
			}

			// -- Auth
			CtxExt(_) => (StatusCode::FORBIDDEN, ClientError::NO_AUTH),

+			// -- Model
+			Model(model::Error::EntityNotFound { entity, id }) => (
+				StatusCode::BAD_REQUEST,
+				ClientError::ENTITY_NOT_FOUND { entity, id: *id },
+			),

            ....

			// -- Fallback.
			_ => (
				StatusCode::INTERNAL_SERVER_ERROR,
				ClientError::SERVICE_ERROR,
			),
		}
	}
}
```

 - 👉 Only `Model(model::Error::EntityNotFound)` will be mapped. Any other `Model(model::Error)` will be ignored and fall through to the fallback.

 **Serialize ClientError to json**

 So far the `ClientError` has been plain named variants but now we have a struct variant. Some minor changes needed to serialize it properly.

 ```diff

#[derive(Debug, 
+        Serialize, 
          strum_macros::AsRefStr)]
+#[serde(tag = "message", content = "detail")]
#[allow(non_camel_case_types)]
pub enum ClientError {
	LOGIN_FAIL,
	NO_AUTH,
    ENTITY_NOT_FOUND { entity: &'static str, id: i64 },
   ...
}
```

What this does is that, when serializing `ENTITY_NOT_FOUND`, the name `ENTITY_NOT_FOUND` is serialized under the json tag: `message` while the content:`{entity, id}` is serialized under `detail`.

**Include this new serialization in the error handlign section of the response mapper**

```diff
	let error_response =
		client_status_error
			.as_ref()
			.map(|(status_code, client_error)| {
				let client_error = to_value(client_error).ok();
+				let message = client_error.as_ref().and_then(|v| v.get("message"));
+				let detail = client_error.as_ref().and_then(|v| v.get("detail"));

				let client_error_body = json!({
					"id": rpc_info.as_ref().map(|rpc| rpc.id.clone()),
					"error": {
+						"message": message, // Variant name
+						"data": {
							"req_uuid": uuid.to_string(),
+							"detail": detail
+						},
					}
				});

				debug!("CLIENT ERROR BODY:\n{client_error_body}");

				// Build the new response from the client_error_body
				(*status_code, Json(client_error_body)).into_response()
			});
```

**Add RPC related logging info**

Create a new struct to hold the needed extra info. These things need to be created very early on to allow for easier debugging.

```rust
[#derive(Debug)]
pub struct RpcInfo {
    pub id: Option<Value>,
    pub method: String,
}
```

He splits the rpc_handler code to allow for the use of this struct without obscuring the core logic

```diff
async fn rpc_handler(
    State(mm): State<ModelManager>,
    ctx: Ctx.
    Json(rpc_req): Json<RpcRequest>,
) -> Response {
+    let rpc_info = RpcInfo {
+        id: rpc_req.id.clone(),
+        method: rpc_req.method.clone(),
+    };

-    _rpc_handler(ctx, mm, rpc_req).await.into_response()
+    let mut res = _rpc_handler(ctx, mm, rpc_req).await.into_response();
+    res.extensions_mut().insert(rpc_info);
+    res
}
```

> Maybe also store it in the request itself earlier on to allow for tracing (_req uuid as well as rpc id_) as well, not just logging.

Now, let the response mapper get this out and log it if present. **Note that, since json-rpc responses should have the same `id` field, for error responses, this is about protocol correctness, not just a nice-to-have** 

```diff
pub async fn mw_reponse_map(
	...
) -> Response {
	....

+	let rpc_info = res.extensions().get::<Arc<RpcInfo>>().map(Arc::as_ref);
    ...
    let client_error_body = json!({
+					"id": rpc_info.as_ref().map(|rpc| rpc.id.clone()),
					"error": {
						"message": message, // Variant name
						"data": {
							"req_uuid": uuid.to_string(),
							"detail": detail
						},
					}
				});
    ....
	let _ = log_request(
		req_method,
		uri,
		req_stamp,
+		rpc_info,
		ctx,
		web_error,
		client_error,
	)
	.await;                    
```

**update logging**

```diff
struct RequestLogLine {
    ...
+   rpc_id: Option<String>,
+   rpc_method: Option<String>
    ...
}
```

 - I'd have thought simply add an `RpcInfo` field to this.
 - However, to allow for `lazy path is the correct path` and avoid sending extra info out in the log-lines, maybe best to explicitly take just the fields that we want to log and not a somewhat opaque `RpcInfo`. What if it starts carrying `user-id` at some later date ?

```diff
pub async fn log_request(
    ...
+   rpc_info: Option<&RpcInfo>,
    ...
) -> Result<()> {
    ...
    let log_line = RequestLogLine {
        ...
        rpc_id : rpc_info.and_then(|rpc| rpc.id.as_ref().map(|id| id.to_string())),
        rpc_methdo : rpc_info.and_then(|rpc| rpc.method.to_string()),
        ...
    }
    ...
}
```

can't we simplify to remove the un-necssary closure in  `rpc.id.as_ref().map(|id| id.to_string()),` to `rpc.id.as_ref().map(to_string()),`


And we are all done with this 4 hour video that took me 2 weeks of on/off with vacations in the middle. 👏 for a superb video dense with great information.

🛑