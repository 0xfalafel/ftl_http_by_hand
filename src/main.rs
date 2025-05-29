use std::{str::FromStr, sync::Arc};

use color_eyre::eyre::eyre;
use nom::Offset;
use rustls::{ClientConfig, KeyLogFile, RootCertStore};
use rustls::pki_types::CertificateDer;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tracing::info;
use tracing_subscriber::{filter::targets::Targets, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> color_eyre::Result<()>{
    color_eyre::install().unwrap();

    let filter_layer = Targets::from_str(std::env::var("RUST_LOG").as_deref().unwrap_or("info")).unwrap();
    let format_layer = tracing_subscriber::fmt::layer();
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(format_layer)
        .init();

    info!("Establishing TCPconnection...");
    let stream = TcpStream::connect("example.com:443").await?;

    info!("Setting up TLS root certificate store");
    let mut roots = rustls::RootCertStore::empty();
    for cert in rustls_native_certs::load_native_certs().expect("could not load platform certs") {
        roots.add(cert).unwrap();
    }

    
    println!("Hello, world!");

    Ok(())
}
