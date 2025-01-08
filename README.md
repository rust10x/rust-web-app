# Rust10x Web App Blueprint for Production Coding

More info at: https://rust10x.com/web-app
Discord: https://discord.gg/XuKWrNGKpC

# Note last commit with `modql 0.4.0-rc.4`

- There is a small change in the `SeaField::new(iden, value)` where the value is now `impl Into<SimpleExpr>`. 
	- `so change:` `SeaField::new(UserIden::Pwd, pwd.into())`
	- `       to:` `SeaField::new(UserIden::Pwd, pwd)`

You can find this change in the `. update to modql 0.4.0-rc.4`

# IMPORTANT NOTE on E06 - 2024-01-23 BIG UPDATE

This update ([GitHub tag: E06](https://github.com/rust10x/rust-web-app/releases/tag/E06)) is significant in many respects:

- **1) Data Model Change**
	- We are transitioning from the simple `Project / Task` model to a more intricate one centered around AI chat, specifically `Agent, Conv / ConvMsg`.
	- Subsequently, we'll introduce `Org / Space` constructs to demonstrate multi-tenancy and a "workspace" type of container, common in many use cases (like GitHub repositories, Discord servers, etc.).
	- The `examples/quick_dev` has been updated to reflect the new data model.
	- IMPORTANT - While `Agent` and `Conv` concepts exist, the blueprint's purpose isn't to develop a complete AI chat system. Instead, it aims to illustrate the common structures needed to build such an application and others. The Agents are merely examples of entities and might later exhibit some "Echo" capability to demonstrate the integration of long-running, event-based services.

- **2) ModelManager DB Transaction Support**
	- There's a significant enhancement to the `ModelManager`, which now contains a `lib_core::model::store::Dbx` implementing an on-demand **database transaction** support.
	- By default, the ModelManager operates non-transactionally; each query executes as its own DB command. However, Bmc functions can transform a ModelManager into a transactional one and initiate/commit a transaction 
		- Search for `mm.dbx().begin_txn()` for an example in `UserBmc::create`.

- **3) Declarative Macros**
	- To reduce boilerplate, this Rust10x blueprint now supports flexible declarative macros (i.e., `macro_rules`) at the `lib_rpc` and `lib_core::model` levels. These create the common basic CRUD JSON-RPC functions and the common BMC CRUD methods.
		- Search for `generate_common_bmc_fns` or `generate_common_rpc_fns` to see them in actions.
	- It's important to note that these declarative macros are additive and optional. In fact, entities can introduce additional behavior as needed or opt out of using these macros if custom logic is required, even for common behaviors.

- **4) Code Update**
	- All JSON-RPC responses now include a `.data` field as `result.data` to represent the requested data. This adds flexibility to later include metadata at the root of the `result` object (the JSON-RPC specification prohibits adding anything at the root of the JSON response).
		- This is in the `lib_rpc::response` crate/module.
	- The introduction of a `conv_id` in the `Ctx` paves the way for a future `Access Control System`, which will be privilege-based and tied to key container constructs (e.g., `Org`, `Space`, `Conv`).

## Rust10x Web App YouTube Videos:

- [Episode 01 - Rust Web App - Base Production Code](https://youtube.com/watch?v=3cA_mk4vdWY&list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q)
    - [Topic video - Code clean -  `#[cfg_attr(...)]` for unit test](https://www.youtube.com/watch?v=DCPs5VRTK-U&list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q)
	- [Topic video - The Reasoning Behind Differentiating ModelControllers and ModelManager](https://www.youtube.com/watch?v=JdLi69mWIIE&list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q)
	- [Topic video - Base64Url - Understanding the Usage and Significance of Base64URL](https://www.youtube.com/watch?v=-9K7zNgsbP0&list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q)

- [Episode 02 - Sea-Query (sql builder) & modql (mongodb like filter)](https://www.youtube.com/watch?v=-dMH9UiwKqg&list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q)

- [Episode 03 - Cargo Workspace (multi-crates)](https://www.youtube.com/watch?v=zUxF0kvydJs&list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q)
	- [AI-Voice-Remastered](https://www.youtube.com/watch?v=iCGIqEWWTcA&list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q)

- [Episode 04 - Multi-Scheme Password Hashing](https://www.youtube.com/watch?v=3E0zK5h9zEs&list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q)

- [Episode 05 - JSON-RPC Dynamic Router](https://www.youtube.com/watch?v=Gc5Nj5LJe1U&list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q)

- **Episode 06 coming upon request on [discord](https://discord.gg/XuKWrNGKpC)**

- Other Related videos: 
	- [Rust Axum Full Course](https://youtube.com/watch?v=XZtlD_m59sM&list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q)


## Starting the DB

```sh
# Start postgresql server docker image:
docker run --rm --name pg -p 5432:5432 \
   -e POSTGRES_PASSWORD=welcome \
   postgres:17

# (optional) To have a psql terminal on pg. 
# In another terminal (tab) run psql:
docker exec -it -u postgres pg psql

# (optional) For pg to print all sql statements.
# In psql command line started above.
ALTER DATABASE postgres SET log_statement = 'all';
```

## Dev (watch)

> NOTE: Install cargo watch with `cargo install cargo-watch`.

```sh
# Terminal 1 - To run the server.
cargo watch -q -c -w crates/services/web-server/src/ -w crates/libs/ -w .cargo/ -x "run -p web-server"

# Terminal 2 - To run the quick_dev.
cargo watch -q -c -w crates/services/web-server/examples/ -x "run -p web-server --example quick_dev"
```

## Dev

```sh
# Terminal 1 - To run the server.
cargo run -p web-server

# Terminal 2 - To run the tests.
cargo run -p web-server --example quick_dev
```

## Unit Test (watch)

```sh
cargo watch -q -c -x "test -- --nocapture"

# Specific test with filter.
cargo watch -q -c -x "test -p lib-core test_create -- --nocapture"
```

## Unit Test

```sh
cargo test -- --nocapture

cargo watch -q -c -x "test -p lib-core model::task::tests::test_create -- --nocapture"
```

## Tools

```sh
cargo run -p gen-key
```

<br />

---

More resources for [Rust for Production Coding](https://rust10x.com)


[This repo on GitHub](https://github.com/rust10x/rust-web-app)