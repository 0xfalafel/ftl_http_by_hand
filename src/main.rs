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
    let stream = TcpStream::connect("example.org:443").await?;

    info!("Setting up TLS root certificate store");
    let mut root_store = rustls::RootCertStore::empty();
    for cert in rustls_native_certs::load_native_certs().expect("could not load platform certs") {
        root_store.add(cert).unwrap();
    }

    let mut client_config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    client_config.key_log = Arc::new(KeyLogFile::new());
    let connector = tokio_rustls::TlsConnector::from(Arc::new(client_config));

    info!("Performing TLS handshake");
    let mut stream = connector.connect("example.org".try_into()?, stream).await?;

    info!("Sending HTTP/1.1 request");
    let req = [
        "GET / HTTP/1.1",
        "Host: example.org",
        "User-Agent: cool-bear/1.0",
        "Connection: close",
        "",
        "",
    ].join("\r\n");
    stream.write_all(req.as_bytes()).await?;

    


    println!("Hello, world!");

    Ok(())
}
