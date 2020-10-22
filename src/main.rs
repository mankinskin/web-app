#![feature(async_closure)]
#![feature(bool_to_option)]
#![feature(map_into_keys_values)]

mod server;
mod shared;
pub use server::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    server::run().await
}
