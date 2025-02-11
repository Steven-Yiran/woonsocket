pub use crate::chunked_tcp_stream::MSG_SIZE_BYTES;
use crate::{
    chunked_tcp_stream::ChunkedTcpStream,
    serialize::{ClientWorkPacket, MessageTrait, ServerWorkPacket},
};
use std::{collections::HashMap, net::TcpStream};

pub mod work_request {
    #[allow(unused_mut)]
    use super::*;

    pub struct ClientWorkPacketConn {
        stream: ChunkedTcpStream,
    }

    impl ClientWorkPacketConn {
        pub fn new(stream: &TcpStream) -> Self {
            let stream = stream.try_clone().expect("Failed to clone stream");
            let chunked_stream = ChunkedTcpStream::new(stream);
            Self { stream: chunked_stream }
        }

        pub fn send_work_msg(
            &mut self,
            work_packet: ClientWorkPacket,
        ) -> Result<(), anyhow::Error> {
            let mut buf = vec![0; MSG_SIZE_BYTES];
            let sz = work_packet.to_bytes(&mut buf)?;
            
            let sz_bytes = (sz as u64).to_be_bytes();
            self.stream.send_msg_chunk(&sz_bytes)?;
            self.stream.send_msg_chunk(&buf[..sz as usize])?;
            
            Ok(())
        }

        pub fn recv_work_msg(&mut self) -> Result<ClientWorkPacket, anyhow::Error> {
            let mut sz_buf = [0; 8];
            self.stream.recv_msg_chunk(&mut sz_buf)?;
            let sz = u64::from_be_bytes(sz_buf);
            
            const MAX_MESSAGE_SIZE: u64 = MSG_SIZE_BYTES as u64;
            if sz > MAX_MESSAGE_SIZE || sz == 0 {
                return Err(anyhow::anyhow!("Invalid message size: {}", sz));
            }
            
            let mut buf = vec![0; sz as usize];
            self.stream.recv_msg_chunk(&mut buf)?;
            
            Ok(ClientWorkPacket::from_bytes(&buf)?)
        }
    }
}

pub mod work_response {
    use super::*;

    pub struct ServerWorkPacketConn {
        stream: ChunkedTcpStream,
    }

    impl ServerWorkPacketConn {
        pub fn new(stream: &TcpStream) -> Self {
            let stream = stream.try_clone().expect("Failed to clone stream");
            let chunked_stream = ChunkedTcpStream::new(stream);
            Self { stream: chunked_stream }
        }

        pub fn send_work_msg(&mut self, packet: ServerWorkPacket) -> Result<(), anyhow::Error> {
            let mut buf = vec![0; MSG_SIZE_BYTES];
            let sz = packet.to_bytes(&mut buf)?;
            
            let sz_bytes = (sz as u64).to_be_bytes();
            self.stream.send_msg_chunk(&sz_bytes)?;
            self.stream.send_msg_chunk(&buf[..sz as usize])?;
            
            Ok(())
        }

        pub fn recv_work_msg(&mut self) -> Result<ServerWorkPacket, anyhow::Error> {
            // Debug size header read
            eprintln!("Attempting to read size header (8 bytes)...");
            let mut sz_buf = [0; 8];
            match self.stream.recv_msg_chunk(&mut sz_buf) {
                Ok(_) => eprintln!("Successfully read size header: {:?}", sz_buf),
                Err(e) => {
                    eprintln!("Failed to read size header: {:?}", e);
                    return Err(e.into());
                }
            }
            
            let sz = u64::from_be_bytes(sz_buf);
            eprintln!("Decoded message size: {} bytes", sz);
            
            // Validate size
            const MAX_MESSAGE_SIZE: u64 = MSG_SIZE_BYTES as u64;
            if sz > MAX_MESSAGE_SIZE || sz == 0 {
                eprintln!("Invalid message size: {} (max: {})", sz, MAX_MESSAGE_SIZE);
                return Err(anyhow::anyhow!("Invalid message size: {}", sz));
            }
            
            // Debug message body read
            eprintln!("Attempting to read message body ({} bytes)...", sz);
            let mut buf = vec![0; sz as usize];
            match self.stream.recv_msg_chunk(&mut buf) {
                Ok(_) => eprintln!("Successfully read message body of {} bytes", sz),
                Err(e) => {
                    eprintln!("Failed to read message body: {:?}", e);
                    return Err(e.into());
                }
            }
            
            // Debug deserialization
            eprintln!("Attempting to deserialize message...");
            match ServerWorkPacket::from_bytes(&buf) {
                Ok(packet) => {
                    eprintln!("Successfully deserialized message");
                    Ok(packet)
                }
                Err(e) => {
                    eprintln!("Failed to deserialize message: {:?}", e);
                    Err(e)
                }
            }
        }
    }
}
