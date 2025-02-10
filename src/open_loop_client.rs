use crate::{
    get_current_time_micros,
    protocol::{work_request::ClientWorkPacketConn, work_response::ServerWorkPacketConn},
    serialize::{ClientWorkPacket, LatencyRecord},
};
use minstant::Instant;
use std::{
    fs::File,
    io::{BufWriter, Write},
    net::{SocketAddrV4, TcpStream},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::app::Work;

fn client_open_loop(
    send_stream: TcpStream,
    thread_start_time: Instant,
    thread_delay: Duration,
    runtime: Duration,
    packets_sent: Arc<AtomicU64>,
    work: Work,
) {
    // TODO: Students will have to write this code.
    // NOTE: It might be helpful to look at protocol.rs first. You'll probably
    // be implementing that alongside this function. If you've done
    // closed_loop_client.rs, then much of the work there applies here too so we
    // recommend working on the closed_loop_client.rs file first.
    //
    // This function is the send side of an open loop client. It sends data
    // every thread-delay duration.
    let mut conn = ClientWorkPacketConn::new(send_stream);
    let start_time = Instant::now();

    while start_time.elapsed() < runtime {
        let work_packet = ClientWorkPacket::new(get_current_time_micros(), work);
        if let Err(e) = conn.send_work_msg(work_packet) {
            eprintln!("Failed to send work packet: {}", e);
            break; // Stop sending if there's an issue
        }
        packets_sent.fetch_add(1, Ordering::SeqCst);

        let elapsed = start_time.elapsed();
        if elapsed < thread_delay {
            let sleep_time = thread_delay - elapsed;
            thread::sleep(sleep_time);
        }
    }
}

fn client_recv_loop(
    recv_stream: TcpStream,
    receiver_complete: Arc<AtomicBool>,
) -> Vec<LatencyRecord> {
    // TODO: Students will have to write this code.
    // This function is the recvs responses for an open loop client.
    let mut conn = ServerWorkPacketConn::new(recv_stream);
    let mut latencies = Vec::new();

    while !receiver_complete.load(Ordering::SeqCst) {
        match conn.recv_work_msg() {
            Ok(server_work_packet) => {
                let recv_timestamp = get_current_time_micros();
                if let Some(latency_record) = server_work_packet.calculate_latency(recv_timestamp) {
                    latencies.push(latency_record);
                } else {
                    eprintln!("Failed to calculate latency");
                }
            }
            Err(e) => {
                if receiver_complete.load(Ordering::SeqCst) {
                    break; // Stop if sending is done and there's no more data
                }
                std::thread::sleep(Duration::from_millis(1)); // Avoid CPU spinning
            }
        }
    }

    latencies
}

fn init_client(
    server_addr: SocketAddrV4,
    thread_delay: Duration,
    runtime: Duration,
    work: Work,
) -> JoinHandle<Vec<LatencyRecord>> {
    let stream = TcpStream::connect(&server_addr).expect("Couldn't connect to server");
    stream.set_nodelay(true).expect("set_nodelay call failed");
    let thread_start_time = Instant::now();

    let sent = Arc::new(AtomicU64::new(0));
    let done = Arc::new(AtomicBool::new(false));

    {
        let stream = stream.try_clone().expect("Failed to clone stream");
        let sent = sent.clone();
        let done = done.clone();
        let _ = thread::spawn(move || {
            client_open_loop(stream, thread_start_time, thread_delay, runtime, sent, work);
            done.store(true, Ordering::SeqCst);
        });
    }

    let recv_handle = {
        let stream = stream.try_clone().expect("Failed to clone stream");
        let done = done.clone();
        thread::spawn(move || client_recv_loop(stream, done))
    };

    recv_handle
}

pub fn run(
    server_addr: SocketAddrV4,
    num_threads: usize,
    interarrival: Duration,
    runtime: Duration,
    work: Work,
    outdir: PathBuf,
) {
    let thread_delay = interarrival * (num_threads as _);

    println!("start: thread_delay {:?}", thread_delay);
    let join_handles: Vec<JoinHandle<Vec<LatencyRecord>>> = (0..num_threads)
        .map(|_| init_client(server_addr, thread_delay, runtime, work))
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
