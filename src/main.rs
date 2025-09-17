use std::{net::ToSocketAddrs, str::FromStr, sync::Arc, time::Instant};

use color_eyre::eyre::eyre;
use nom::Offset;
use rustls::{ClientConfig, KeyLogFile};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tracing::info;
use tracing_subscriber::{filter::targets::Targets, layer::SubscriberExt, util::SubscriberInitExt};

mod http11;

#[tokio::main]
async fn main() -> color_eyre::Result<()>{
    color_eyre::install().unwrap();

    let filter_layer = Targets::from_str(std::env::var("RUST_LOG").as_deref().unwrap_or("info")).unwrap();
    let format_layer = tracing_subscriber::fmt::layer();
    tracing_subscriber::registry()
        .with(filter_layer)
        .with(format_layer)
        .init();

    info!("Performing DNS lookup...");
    let before = Instant::now();
    let addr = "example.com:443"
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| eyre!("Failed to resolve address for example.com:443"))?;
    println!("{:?} DNS lookup ", before.elapsed());

    info!("Establishing TCPconnection...");
    let before = Instant::now();
    let stream = TcpStream::connect(addr).await?;
    println!("{:?} TCP connect", before.elapsed());

    info!("Setting up TLS root certificate store");
    let mut root_store = rustls::RootCertStore::empty();
    for cert in rustls_native_certs::load_native_certs()
        .expect("could not load platform certs") {
            root_store.add(cert).unwrap();
    }

    let mut client_config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    client_config.key_log = Arc::new(KeyLogFile::new());
    let connector = tokio_rustls::TlsConnector::from(
        Arc::new(client_config)
    );

    info!("Performing TLS handshake");
    let before = Instant::now();
    let mut stream = connector
        .connect("example.org".try_into()?, stream).await?;
    println!("{:?} TLS handshake", before.elapsed());

    info!("Sending HTTP/1.1 request");
    let before = Instant::now();
    let req = [
        "GET / HTTP/1.1",
        "Host: example.org",
        "User-Agent: cool-bear/1.0",
        "Connection: close",
        "",
        "",
    ].join("\r\n");
    stream.write_all(req.as_bytes()).await?;
    println!("{:?} Request send", before.elapsed());

    info!("Reading HTTP/1.1 response");
    let before = Instant::now();
    let mut accum : Vec<u8> = Default::default();
    let mut rd_buf = [0u8; 1024];

    let (body_offest, res) = loop {
        let n = stream.read(&mut rd_buf[..]).await?;
        info!("Reading {n} bytes");

        if n == 0 {
            return Err(eyre!(
                "Unexpected EOF (server closed connection during headers)"
            ));
        }

        accum.extend_from_slice(&rd_buf[..n]);

        match http11::response(&accum) {
            Err(e) => {
                if e.is_incomplete() {
                    info!("Need to read more, continuing");
                    continue;
                } else {
                    return Err(eyre!("parse error: {e}"));
                }
            },
            Ok((remain, res)) => {
                let body_offset = accum.offset(remain);
                break (body_offset, res);
            }
        }
    };
    println!("{:?} Response header read", before.elapsed());

    info!("Got HTTP1/1 response: {:#?}", res);
    let before = Instant::now();
    let mut body_accum = accum[body_offest..].to_vec();
    // header names are case-insensitive, let's get it right. we're assuming
    // that the absence of content-length means there's no body, and we also
    // don't support chunked transfer encoding.
    let content_length = res
        .headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
        .map(|(_, v)| v.parse::<usize>().unwrap())
        .unwrap_or_default();
    
    while body_accum.len() < content_length {
        let n = stream.read(&mut rd_buf[..]).await?;
        info!("Read {n} bytes");
        if n == 0 {
            return Err(eyre!("unexpected EOF (peer closed connection during body)"))
        }

        body_accum.extend_from_slice(&rd_buf[..n]);
    }
    println!("{:?} Response body read", before.elapsed());
    
    info!("===== Response body =====");
    info!("{}", String::from_utf8_lossy(&body_accum));

    Ok(())
}
