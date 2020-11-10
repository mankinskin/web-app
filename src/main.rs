#![feature(async_closure)]
#![feature(bool_to_option)]
#![feature(map_into_keys_values)]

mod server;
mod shared;
pub use server::*;

#[cfg(all(feature = "actix_server", feature = "tide_server"))]
compile_error!("features [`tide_server`, `actix_server`] are mutually exclusive");

#[cfg(feature = "actix_server")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    server::run().await
}

#[cfg(feature = "tide_server")]
#[tokio::main]
async fn main() -> std::io::Result<()> {
    server::run().await
}
