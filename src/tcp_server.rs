use crate::{
    protocol::{work_request::ClientWorkPacketConn, work_response::ServerWorkPacketConn},
    serialize::ClientWorkPacket,
};

use std::{
    net::{SocketAddrV4, TcpListener, TcpStream},
    sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}},
    thread,
    time::{Duration, Instant},
};

// Struct to track server load metrics
struct ServerLoadTracker {
    received_requests: AtomicUsize,
    completed_requests: AtomicUsize,
    start_time: Instant,
}

impl ServerLoadTracker {
    fn new() -> Self {
        ServerLoadTracker {
            received_requests: AtomicUsize::new(0),
            completed_requests: AtomicUsize::new(0),
            start_time: Instant::now(),
        }
    }

    fn record_received(&self) {
        self.received_requests.fetch_add(1, Ordering::SeqCst);
    }

    fn record_completed(&self) {
        self.completed_requests.fetch_add(1, Ordering::SeqCst);
    }

    fn get_offered_load(&self) -> f64 {
        let elapsed_secs = self.start_time.elapsed().as_secs_f64();
        if elapsed_secs > 0.0 {
            self.received_requests.load(Ordering::SeqCst) as f64 / elapsed_secs
        } else {
            0.0
        }
    }

    fn get_achieved_load(&self) -> f64 {
        let elapsed_secs = self.start_time.elapsed().as_secs_f64();
        if elapsed_secs > 0.0 {
            self.completed_requests.load(Ordering::SeqCst) as f64 / elapsed_secs
        } else {
            0.0
        }
    }

    fn print_metrics(&self) {
        let received = self.received_requests.load(Ordering::SeqCst);
        let completed = self.completed_requests.load(Ordering::SeqCst);
        let offered_load = self.get_offered_load();
        let achieved_load = self.get_achieved_load();
        let elapsed_secs = self.start_time.elapsed().as_secs_f64();
        
        println!("\nServer Load Metrics:");
        println!("Runtime: {:.2} seconds", elapsed_secs);
        println!("Total received requests: {}", received);
        println!("Total completed requests: {}", completed);
        println!("Offered load: {:.2} req/s", offered_load);
        println!("Achieved load: {:.2} req/s", achieved_load);
        
        if received > 0 {
            println!("Completion rate: {:.2}%", (completed as f64 / received as f64) * 100.0);
        }
    }
}

pub fn tcp_server(addr: SocketAddrV4) -> Result<(), anyhow::Error> {
    let listener = TcpListener::bind(addr).unwrap();
    let load_tracker = Arc::new(ServerLoadTracker::new());
    
    // Periodically print metrics
    let tracker_clone = Arc::clone(&load_tracker);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(10));
            tracker_clone.print_metrics();
        }
    });
    
    println!("Server started on {:?}", addr);
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let tracker_clone = Arc::clone(&load_tracker);
                thread::spawn(move || {
                    if let Err(e) = handle_conn(stream, tracker_clone) {
                        eprintln!("Connection handler error: {:?}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {:?}", e);
            }
        }
    }

    Ok(())
}

fn handle_conn(stream: TcpStream, load_tracker: Arc<ServerLoadTracker>) -> Result<(), anyhow::Error> {
    let peer_addr = stream.peer_addr()?;
    eprintln!("Starting to handle connection from {:?}", peer_addr);
    
    let mut client_conn = ClientWorkPacketConn::new(&stream);
    let mut server_conn = ServerWorkPacketConn::new(&stream);
    
    loop {
        // Record that we're about to receive a request
        load_tracker.record_received();
        
        let work_packet = match client_conn.recv_work_msg() {
            Ok(packet) => {
                packet
            }
            Err(e) => {
                return Err(e);
            }
        };
        
        let server_work_packet = work_packet.do_work();
        
        if let Err(e) = server_conn.send_work_msg(server_work_packet) {
            return Err(e);
        }
        
        // Record that we've completed processing a request
        load_tracker.record_completed();
    }
}
