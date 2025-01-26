use clap::Parser;
use netapis_s25_dev::{app::Work, closed_loop_client, open_loop_client};
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    path::PathBuf,
    time::Duration,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
pub struct Opt {
    #[arg(long, help = "Specify this argument for an open loop client")]
    interval_us: Option<u64>,

    #[arg(short, long)]
    num_threads: u64,

    #[arg(short, long)]
    runtime_secs: u64,

    #[arg(short, long)]
    ip: Ipv4Addr,

    #[arg(short, long)]
    port: u16,

    #[arg(short, long)]
    work: Work,

    #[arg(short, long)]
    outpath: PathBuf,
}

fn main() {
    let opt = Opt::parse();
    let server_addr = SocketAddrV4::new(opt.ip, opt.port);
    let runtime = Duration::from_secs(opt.runtime_secs);
    let outpath = opt.outpath.clone();
    if let Some(interarrival) = opt.interval_us {
        open_loop_client::run(
            server_addr,
            opt.num_threads as _,
            Duration::from_micros(interarrival),
            runtime,
            opt.work,
            outpath,
        );
    } else {
        closed_loop_client::run(
            server_addr,
            opt.num_threads as _,
            runtime,
            opt.work,
            outpath,
        );
    }
}
