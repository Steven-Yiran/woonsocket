//! Message serialization types and functions.

use crate::{app::Work, get_current_time_micros};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct LatencyRecord {
    pub latency: u64,
    pub send_timestamp: u64,
    pub server_processing_time: u64,
    pub recv_timestamp: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ClientWorkPacket {
    id: u64,
    work: Work,
    timestamp: u64,
}

impl ClientWorkPacket {
    // Note: We generate a client time here to encapsulate calculating
    // latency from student work.
    pub fn new(id: u64, work: Work) -> Self {
        Self {
            id,
            work,
            timestamp: get_current_time_micros(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn do_work(&self) -> ServerWorkPacket {
        let start = Instant::now();
        let payload = self.work.perform();
        let dur = start.elapsed().as_micros() as u64;
        ServerWorkPacket {
            status: ServerWorkStatus::Completed,
            server_processing_time: dur,
            client_id: self.id,
            client_send_time: self.timestamp,
            payload,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerWorkPacket {
    status: ServerWorkStatus,
    server_processing_time: u64,
    client_id: u64,
    client_send_time: u64,
    payload: Option<Vec<u8>>,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ServerWorkStatus {
    Completed = 0,
    Failed = 1,
}

impl ServerWorkPacket {
    pub fn client_id(&self) -> u64 {
        self.client_id
    }

    pub fn client_send_time(&self) -> u64 {
        self.client_send_time
    }

    // Note: We calculate latency here to separate this out from student work
    pub fn calculate_latency(&self, receive_time: u64) -> Option<LatencyRecord> {
        match self.status {
            ServerWorkStatus::Completed => {
                if receive_time < self.client_send_time {
                    eprintln!("Warning: Timestamp inconsistency detected!");
                    eprintln!("  Send time: {}", self.client_send_time);
                    eprintln!("  Receive time: {}", receive_time);
                    return None;
                }
                let rtt = receive_time - self.client_send_time;
                let processing_time = self.server_processing_time;
                let actual_latency = (rtt - processing_time) / 2;

                Some(LatencyRecord {
                    latency: actual_latency,
                    send_timestamp: self.client_send_time,
                    server_processing_time: self.server_processing_time,
                    recv_timestamp: receive_time,
                })
            }
            ServerWorkStatus::Failed => None,
        }
    }
}

pub fn generate_random_work() -> Work {
    let mut rng = rand::thread_rng();
    match rng.gen_range(0..6) {
        0 => Work::Immediate,
        1 => Work::Const(rng.gen_range(1..100)),
        2 => Work::Poisson(NonZeroU64::new(rng.gen_range(1..100)).unwrap()),
        3 => Work::BusyTimeConst(rng.gen_range(1..100)),
        4 => Work::BusyWorkConst(rng.gen_range(1..100)),
        _ => Work::Immediate,
    }
}

pub trait MessageTrait: Serialize + for<'a> Deserialize<'a> {
    fn to_bytes(&self, buf: &mut [u8]) -> Result<u64, anyhow::Error> {
        let sz = bincode::serialized_size(self)?;
        bincode::serialize_into(buf, self)?;
        Ok(sz)
    }

    fn to_vec(&self, buf: &mut Vec<u8>) -> Result<(), anyhow::Error> {
        bincode::serialize_into(buf, self)?;
        Ok(())
    }

    fn from_bytes(buf: &[u8]) -> Result<Self, anyhow::Error> {
        let packet: Self = bincode::deserialize_from(buf)?;
        Ok(packet)
    }
}

impl MessageTrait for ClientWorkPacket {}
impl MessageTrait for ServerWorkPacket {}
