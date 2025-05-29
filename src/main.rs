use std::{str::FromStr, sync::Arc};

use color_eyre::eyre::eyre;
use nom::Offset;
use rustls::{ClientConfig, KeyLogFile, RootCertStore};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tracing::info;
use tracing_subscriber::{filter::targets::Targets, layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    println!("Hello, world!");
}
