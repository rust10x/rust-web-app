# Rust10x Web App Blueprint for Production Coding

More info at: https://rust10x.com/web-app

## Rust10x Web App YouTube Videos:

- [Episode 01 - Rust Web App - Course to Production Coding](https://youtube.com/watch?v=3cA_mk4vdWY&list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q)
    - [Topic video - Code clean -  `#[cfg_attr(...)]` for unit test](https://www.youtube.com/watch?v=JdLi69mWIIE&list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q)
	- [Topic video - The Reasoning Behind Differentiating ModelControllers and ModelManager](https://www.youtube.com/watch?v=JdLi69mWIIE&list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q)
	- [Topic video - Base64Url - Understanding the Usage and Significance of Base64URL](https://www.youtube.com/watch?v=-9K7zNgsbP0&list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q)
- Other Related videos: 
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