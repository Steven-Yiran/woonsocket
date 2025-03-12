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

// Simple struct to track attempted load
struct AttemptedLoadTracker {
    request_count: usize,
    start_time: Instant,
}

impl AttemptedLoadTracker {
    fn new() -> Self {
        AttemptedLoadTracker {
            request_count: 0,
            start_time: Instant::now(),
        }
    }

    fn record_attempt(&mut self) {
        self.request_count += 1;
    }

    fn get_attempted_load(&self) -> f64 {
        let elapsed_secs = self.start_time.elapsed().as_secs_f64();
        if elapsed_secs > 0.0 {
            self.request_count as f64 / elapsed_secs
        } else {
            0.0
        }
    }
}

fn client_worker(server_addr: SocketAddrV4, runtime: Duration, work: Work) -> (Vec<LatencyRecord>, AttemptedLoadTracker) {
    let mut latencies = Vec::new();
    let mut load_tracker = AttemptedLoadTracker::new();
    let start = Instant::now();
    while start.elapsed().as_secs() < runtime.as_secs() {
        let stream = TcpStream::connect(&server_addr).expect("Failed to connect to server");
        let mut client_conn = ClientWorkPacketConn::new(&stream);
        let mut server_conn = ServerWorkPacketConn::new(&stream);

        let work_packet = ClientWorkPacket::new(rand::random(), work);
        
        // Record attempt before sending
        load_tracker.record_attempt();
        
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
    
    (latencies, load_tracker)
}

pub fn init_client(
    server_addr: SocketAddrV4,
    runtime: Duration,
    work: Work,
) -> JoinHandle<(Vec<LatencyRecord>, AttemptedLoadTracker)> {
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

    // Collect latencies and load metrics
    let mut total_attempts = 0;
    let mut total_runtime_secs = 0.0;
    let mut thread_loads = Vec::new();
    let mut median_latencies = Vec::new();
    let mut p95_latencies = Vec::new();
    let mut p99_latencies = Vec::new();
    
    // Define warm-up constant to ignore initial records for more accurate measurements
    const WARM_UP: usize = 50;

    for (i, handle) in join_handles.into_iter().enumerate() {
        let (thread_latencies, load_tracker) = handle.join().unwrap();
        
        let attempted_load = load_tracker.get_attempted_load();
        thread_loads.push(attempted_load);
        
        // Accumulate metrics
        total_attempts += load_tracker.request_count;
        total_runtime_secs += load_tracker.start_time.elapsed().as_secs_f64();
        
        // Calculate percentile latencies for this thread, ignoring warm-up records
        if thread_latencies.len() > WARM_UP {
            let mut latency_values: Vec<u64> = thread_latencies.iter()
                .skip(WARM_UP)  // Skip the first WARM_UP records
                .map(|record| record.latency)
                .collect();
            latency_values.sort();
            
            let median_idx = latency_values.len() / 2;
            let p95_idx = (latency_values.len() as f64 * 0.95) as usize;
            let p99_idx = (latency_values.len() as f64 * 0.99) as usize;
            
            median_latencies.push(latency_values[median_idx]);
            p95_latencies.push(latency_values[p95_idx]);
            p99_latencies.push(latency_values[p99_idx]);
        }  
    }
    
    // Calculate aggregate attempted load
    let avg_runtime = total_runtime_secs / num_threads as f64;
    let aggregate_attempted_load = if avg_runtime > 0.0 { 
        total_attempts as f64 / avg_runtime 
    } else { 
        0.0 
    };
    
    println!("\nAggregate Metrics:");
    println!("Total attempted requests: {}", total_attempts);
    println!("Attempted load: {:.2} req/s", aggregate_attempted_load);
    println!("Average attempted load per thread: {:.2} req/s", 
             if !thread_loads.is_empty() { 
                 thread_loads.iter().sum::<f64>() / thread_loads.len() as f64 
             } else { 
                 0.0 
             });

    // Output mean aggregated latencies
    let mean_median_latency = median_latencies.iter().sum::<u64>() as f64 / median_latencies.len() as f64;
    let mean_p95_latency = p95_latencies.iter().sum::<u64>() as f64 / p95_latencies.len() as f64;
    let mean_p99_latency = p99_latencies.iter().sum::<u64>() as f64 / p99_latencies.len() as f64;

    println!("\nMean Aggregated Latencies:");
    println!("Median latency: {:.2} us", mean_median_latency);
    println!("95th percentile latency: {:.2} us", mean_p95_latency);
    println!("99th percentile latency: {:.2} us", mean_p99_latency);
}
