//! Server logic for the CS1675 network APIs project.

use clap::{Parser, ValueEnum};
use netapis_s25_dev::tcp_server::tcp_server;

use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;

#[derive(Copy, Clone, Debug, ValueEnum, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum ServerKind {
    tcp,
}

impl ServerKind {
    pub fn as_string_arg(&self) -> String {
        match self {
            Self::tcp => "tcp",
        }
        .into()
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    #[arg(short, long)]
    port: u16,

    #[arg(short, long)]
    kind: ServerKind,

    #[arg(short, long)]
    runtime_secs: u64,
}

fn main() {
    let args = Args::parse();
    let addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, args.port);
    let runtime_secs = args.runtime_secs;

    std::thread::spawn(move || match args.kind {
        ServerKind::tcp => tcp_server(addr),
    });
    std::thread::sleep(Duration::from_secs(runtime_secs + 1));
}
