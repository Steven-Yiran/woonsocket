//! CS1675 network APIs project.

pub mod app;
pub mod chunked_tcp_stream;
pub mod closed_loop_client;
pub mod open_loop_client;
pub mod protocol;
pub mod serialize;
pub mod tcp_server;

pub fn get_current_time_micros() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros() as u64
}
