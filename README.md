# WARNING - THIS BRANCH (nx-e02-tmp) IS A TEMPORARY BRANCH 

- This branch is being prepared for the next episode of the [rust10x web-app production blueprint](https://rust10x/web-app).
- Once the episode is released:
	- The `main` branch will be updated with the corresponding chapter commits (which will not originate from this branch, but will have the same end code).
	- This branch will be deleted.

# Rust10x Web App Blueprint for Production Coding

More info at: https://rust10x.com/web-app

- rust-web-app YouTube episodes:
	- [Episode 01 - Rust Web App - Course to Production Coding](https://youtube.com/watch?v=3cA_mk4vdWY&list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q)
- Related videos: 
	- [Rust Axum Full Course](https://youtube.com/watch?v=XZtlD_m59sM&list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q)

## Starting the DB

```sh
# Start postgresql server docker image:
docker run --rm --name pg -p 5432:5432 \
   -e POSTGRES_PASSWORD=welcome \
   postgres:15

# (optional) To have a psql terminal on pg. 
# In another terminal (tab) run psql:
docker exec -it -u postgres pg psql

# (optional) For pg to print all sql statements.
# In psql command line started above.
ALTER DATABASE postgres SET log_statement = 'all';
```


## Various Cargo run

- Web-Server
	- `cargo run -p web-server` - Start the web server
	- `cargo run -p web-server --example quick_dev` - Run the quick_dev main file. 
- lib_core test
	- `cargo test -p lib`

- Tools
	- `cargo run -p gen_key` - To genreate a new key (in Base64 URL)
	
## Dev (watch)

> NOTE: Install cargo watch with `cargo install cargo-watch`.

```sh
# Terminal 1 - To run the server.
cargo watch -q -c -w src/ -w .cargo/ -x "run"

# Terminal 2 - To run the quick_dev.
cargo watch -q -c -w examples/ -x "run --example quick_dev"
```

## Unit Test (watch)

```sh
cargo watch -q -c -x "test -- --nocapture"

# Specific test with filter.
cargo watch -q -c -x "test model::task::tests::test_create -- --nocapture"
```

## Dev

```sh
# Terminal 1 - To run the server.
cargo run

# Terminal 2 - To run the tests.
cargo run --example quick_dev
```

## Unit Test

```sh
cargo test -- --nocapture

cargo watch -q -c -x test model::task::tests::test_create -- --nocapture
```

<br />

---

More resources for [Rust for Production Coding](https://rust10x.com)


[This repo on GitHub](https://github.com/rust10x/rust-web-app)