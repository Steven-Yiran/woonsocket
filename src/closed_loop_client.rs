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
    let stream = TcpStream::connect(&server_addr).expect("Failed to connect to server");
    let mut client_conn = ClientWorkPacketConn::new(&stream);
    let mut server_conn = ServerWorkPacketConn::new(&stream);

    let start = Instant::now();
    while start.elapsed() < runtime {
        let work_packet = ClientWorkPacket::new(rand::random(), work);        
        // Send the work packet to the server
        if let Err(e) = client_conn.send_work_msg(work_packet) {
            eprintln!("Failed to send work packet: {:?}", e);
            continue;
        }
        // Receive the server's response
        let server_work_packet = match server_conn.recv_work_msg() {
            Ok(packet) => packet,
            Err(e) => {
                eprintln!("Failed to receive server work packet: {:?}", e);
                continue;
            }
        };
        // Calculate latency
        let recv_timestamp = get_current_time_micros();
        if let Some(latency_record) = server_work_packet.calculate_latency(recv_timestamp) {
            latencies.push(latency_record);
        }
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
        // print the length of the thread_latencies
        println!("Thread latency length: {}", thread_latencies.len());
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
