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

// Struct to track attempted load
struct AttemptedLoadTracker {
    packets_sent: Arc<AtomicU64>,
    start_time: Instant,
}

impl AttemptedLoadTracker {
    fn new(packets_sent: Arc<AtomicU64>) -> Self {
        AttemptedLoadTracker {
            packets_sent,
            start_time: Instant::now(),
        }
    }

    fn get_attempted_load(&self) -> f64 {
        let elapsed_secs = self.start_time.elapsed().as_secs_f64();
        if elapsed_secs > 0.0 {
            self.packets_sent.load(Ordering::SeqCst) as f64 / elapsed_secs
        } else {
            0.0
        }
    }
}

fn client_open_loop(
    send_stream: TcpStream,
    thread_start_time: Instant,
    thread_delay: Duration,
    runtime: Duration,
    packets_sent: Arc<AtomicU64>,
    work: Work,
) {
    let mut conn = ClientWorkPacketConn::new(&send_stream);
    let mut next_send_time = thread_start_time;

    while thread_start_time.elapsed() < runtime {
        let work_packet = ClientWorkPacket::new(get_current_time_micros(), work);
        if conn.send_work_msg(work_packet).is_ok() {
            packets_sent.fetch_add(1, Ordering::SeqCst);
            next_send_time += thread_delay;
            // Use spin lock instead of thread::sleep
            while Instant::now() < next_send_time {
                std::hint::spin_loop();
            }
        } else {
            break;
        }
    }
}

fn client_recv_loop(
    recv_stream: TcpStream,
    receiver_complete: Arc<AtomicBool>,
) -> Vec<LatencyRecord> {
    let mut conn = ServerWorkPacketConn::new(&recv_stream);
    let mut latencies = Vec::new();

    // Set a reasonable timeout
    recv_stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
    
    while !receiver_complete.load(Ordering::SeqCst) {
        match conn.recv_work_msg() {
            Ok(server_work_packet) => {
                let recv_timestamp = get_current_time_micros();
                if let Some(latency_record) = server_work_packet.calculate_latency(recv_timestamp) {
                    latencies.push(latency_record);
                }
            }
            Err(e) => {
                // Convert anyhow::Error to std::io::Error if possible
                if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
                    // Don't break on timeouts
                    if io_err.kind() == std::io::ErrorKind::WouldBlock || 
                       io_err.kind() == std::io::ErrorKind::TimedOut {
                        continue;
                    }
                    
                    // Only break on connection-fatal errors
                    if io_err.kind() == std::io::ErrorKind::ConnectionReset ||
                       io_err.kind() == std::io::ErrorKind::ConnectionAborted ||
                       io_err.kind() == std::io::ErrorKind::BrokenPipe {
                        eprintln!("Connection error: {:?}", io_err);
                        break;
                    }
                }
                
                // For other errors, log and continue collecting
                eprintln!("Error receiving work packet: {:?}", e);
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
) -> (JoinHandle<Vec<LatencyRecord>>, Arc<AtomicU64>) {
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

    (recv_handle, sent)
}

pub fn run(
    server_addr: SocketAddrV4,
    num_threads: usize,
    interarrival: Duration,
    runtime: Duration,
    work: Work,
    outdir: PathBuf,
) {    
    // Initialize clients and collect handles and packet counters
    let mut join_handles = Vec::new();
    let mut packet_counters = Vec::new();
    
    for i in 0..num_threads {
        let (handle, packets_sent) = init_client(server_addr, interarrival, runtime, work);
        join_handles.push(handle);
        packet_counters.push(packets_sent);
    }

    // Create load trackers for each thread
    let load_trackers: Vec<_> = packet_counters.iter()
        .map(|counter| AttemptedLoadTracker::new(counter.clone()))
        .collect();

    // Collect latencies
    let mut request_latencies: Vec<Vec<LatencyRecord>> = Vec::new();
    for handle in join_handles {
        let thread_latencies = handle.join().unwrap();
        request_latencies.push(thread_latencies);
    }

    // Calculate and print load metrics
    let mut thread_loads = Vec::new();
    let mut total_packets = 0;
    
    for (i, tracker) in load_trackers.iter().enumerate() {
        let attempted_load = tracker.get_attempted_load();
        thread_loads.push(attempted_load);
        
        let packets = tracker.packets_sent.load(Ordering::SeqCst);
        total_packets += packets;
        
        println!("Thread {} latency count: {}", i, request_latencies[i].len());
        println!("Thread {} packets sent: {}", i, packets);
        println!("Thread {} attempted load: {:.2} req/s", i, attempted_load);
    }
    
    // Calculate aggregate metrics
    let avg_runtime = load_trackers.iter()
        .map(|tracker| tracker.start_time.elapsed().as_secs_f64())
        .sum::<f64>() / num_threads as f64;
    
    let aggregate_attempted_load = if avg_runtime > 0.0 {
        total_packets as f64 / avg_runtime
    } else {
        0.0
    };
    
    println!("\nAggregate Metrics:");
    println!("Total packets sent: {}", total_packets);
    println!("Attempted load: {:.2} req/s", aggregate_attempted_load);
    println!("Average attempted load per thread: {:.2} req/s", 
             if !thread_loads.is_empty() { 
                 thread_loads.iter().sum::<f64>() / thread_loads.len() as f64 
             } else { 
                 0.0 
             });
    
    // Calculate latency percentiles if we have enough data
    if request_latencies.iter().any(|latencies| !latencies.is_empty()) {
        // Define warm-up constant to ignore initial records for more accurate measurements
        const WARM_UP: usize = 50;
        
        let mut median_latencies = Vec::new();
        let mut p95_latencies = Vec::new();
        let mut p99_latencies = Vec::new();
        
        for thread_latencies in &request_latencies {
            if thread_latencies.len() > WARM_UP {
                let mut latency_values: Vec<u64> = thread_latencies.iter()
                    .skip(WARM_UP)  // Skip the first WARM_UP records
                    .map(|record| record.latency)
                    .collect();
                latency_values.sort();
                
                if !latency_values.is_empty() {
                    let median_idx = latency_values.len() / 2;
                    let p95_idx = (latency_values.len() as f64 * 0.95) as usize;
                    let p99_idx = (latency_values.len() as f64 * 0.99) as usize;
                    
                    median_latencies.push(latency_values[median_idx]);
                    p95_latencies.push(latency_values[p95_idx]);
                    p99_latencies.push(latency_values[p99_idx]);
                }
            }
        }
        
        if !median_latencies.is_empty() {
            let mean_median_latency = median_latencies.iter().sum::<u64>() as f64 / median_latencies.len() as f64;
            let mean_p95_latency = p95_latencies.iter().sum::<u64>() as f64 / p95_latencies.len() as f64;
            let mean_p99_latency = p99_latencies.iter().sum::<u64>() as f64 / p99_latencies.len() as f64;
            
            println!("\nMean Aggregated Latencies:");
            println!("Median latency: {:.2} us", mean_median_latency);
            println!("95th percentile latency: {:.2} us", mean_p95_latency);
            println!("99th percentile latency: {:.2} us", mean_p99_latency);
        }
    }
}
