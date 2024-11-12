# Install Rust - WSL 
from https://www.rust-lang.org/tools/install

`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` and then respond to questions to install the default 

# cargo aditions

 - `cargo install cargo-binstall`  _installs from source by compiling_
 - `cargo binstall cargo-watch` _much faster binary install_

# VSCode extensions

Install the `rust-analyzer` extension. This is made by the rust language group: this is what provides intellisense etc for the projects. 
 - This expects a `Cargo.toml` at the top-level of the workspace
 - Auto-discovers `Cargo.toml` files one level deep 