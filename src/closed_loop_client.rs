use crate::{
    app::Work,
    get_current_time_micros,
    protocol::{work_request::ClientWorkPacketConn, work_response::ServerWorkPacketConn},
    serialize::{ClientWorkPacket, LatencyRecord},
};
use std::{
    fs::File,
    io::{BufWriter, Write},
    net::{SocketAddrV4, TcpStream},
    path::PathBuf,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

fn client_worker(server_addr: SocketAddrV4, runtime: Duration, work: Work) -> Vec<LatencyRecord> {
    // TODO: Students will have to write this code.
    // NOTE: It might be helpful to look at protocol.rs first. You'll probably
    // be implementing that alongside this function.
    //
    // This function is a closed loop client sending a request, then waiting for
    // a response. It should return a vector of latency records.
    
    let mut latencies = Vec::new();
    let stream = TcpStream::connect(&server_addr).except("Failed to connect to server");
    let stream = stream.try_clone().except("Failed to clone stream");
    let mut client_conn = ClientWorkPacketConn::new(stream);
    let stream = stream.try_clone().except("Failed to clone stream");
    let mut server_conn = ServerWorkPacketConn::new(stream);

    let start = Instant::now();
    while start.elapsed() < runtime {
        let work_packet = ClientWorkPacket::new(rand::random(), work);
        let send_timestamp = get_current_time_micros();
        client_conn.send_work_msg(work_packet).unwrap();
        let server_work_packet = server_conn.recv_work_msg().unwrap();
        let recv_timestamp = get_current_time_micros();
        let latency = recv_timestamp - send_timestamp;
        latencies.push(LatencyRecord {
            latency,
            send_timestamp,
            server_processing_time: server_work_packet.server_processing_time,
            recv_timestamp,
        });
    }
    latencies
}

pub fn init_client(
    server_addr: SocketAddrV4,
    runtime: Duration,
    work: Work,
) -> JoinHandle<Vec<LatencyRecord>> {
    thread::spawn(move || client_worker(server_addr, runtime, work))
}

pub fn run(
    server_addr: SocketAddrV4,
    num_threads: usize,
    runtime: Duration,
    work: Work,
    outdir: PathBuf,
) {
    let join_handles: Vec<_> = (0..num_threads)
        .map(|_| init_client(server_addr, runtime, work))
        .collect();

    // Collect latencies
    let mut request_latencies: Vec<Vec<LatencyRecord>> = Vec::new();
    for handle in join_handles {
        let thread_latencies = handle.join().unwrap();
        request_latencies.push(thread_latencies);
    }

    // TODO: Output your request latencies to make your graph. You can calculate
    // your graph data here, or output raw data and calculate them externally.
    // You SHOULD write your output to outdir.
    let mut output_file = BufWriter::new(File::create(outdir.join("latencies.txt")).unwrap());
    for thread_latencies in request_latencies {
        for latency in thread_latencies {
            writeln!(
                output_file,
                "{} {} {} {}",
                latency.latency,
                latency.send_timestamp,
                latency.server_processing_time,
                latency.recv_timestamp
            )
            .unwrap();
        }
    }

}
