#![expect(dead_code, unused_imports, clippy::todo, clippy::unused_async)]
mod client;
mod codec;
mod linestream;
mod messages;
mod util;

fn main() {
    println!("Hello, world!");
}
