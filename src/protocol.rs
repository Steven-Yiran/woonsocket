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
            // First serialize the packet to get its size
            let mut buf = vec![0; MSG_SIZE_BYTES];
            let sz = work_packet.to_bytes(&mut buf)?;
            
            eprintln!("Sending work message of size {} bytes", sz);
            
            // Convert size to big-endian bytes and send
            let sz_bytes = (sz as u64).to_be_bytes();
            eprintln!("Sending size header: {:?}", sz_bytes);
            self.stream.send_msg_chunk(&sz_bytes)?;
            
            // Send the actual message
            eprintln!("Sending message payload");
            self.stream.send_msg_chunk(&buf[..sz as usize])?;
            
            eprintln!("Message sent successfully");
            Ok(())
        }

        pub fn recv_work_msg(&mut self) -> Result<ClientWorkPacket, anyhow::Error> {
            eprintln!("Starting to receive client work message...");
            
            // Read message size
            let mut sz_buf = [0; 8];
            self.stream.recv_msg_chunk(&mut sz_buf)?;
            let sz = u64::from_be_bytes(sz_buf);
            eprintln!("Received size header: {} bytes", sz);
            
            // Validate message size
            const MAX_MESSAGE_SIZE: u64 = MSG_SIZE_BYTES as u64;
            if sz > MAX_MESSAGE_SIZE {
                return Err(anyhow::anyhow!(
                    "Message size too large: {} bytes (max: {})",
                    sz,
                    MAX_MESSAGE_SIZE
                ));
            }
            
            // Read the message
            let mut buf = vec![0; sz as usize];
            self.stream.recv_msg_chunk(&mut buf)?;
            eprintln!("Received message payload");
            
            // Deserialize the message
            let packet = ClientWorkPacket::from_bytes(&buf)?;
            eprintln!("Successfully deserialized client work packet");
            
            Ok(packet)
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
            
            eprintln!("Sending server response of size {} bytes", sz);
            
            // Send size as big-endian bytes
            let sz_bytes = (sz as u64).to_be_bytes();
            eprintln!("Sending size header: {:?}", sz_bytes);
            self.stream.send_msg_chunk(&sz_bytes)?;
            
            // Send the message
            eprintln!("Sending message payload");
            self.stream.send_msg_chunk(&buf[..sz as usize])?;
            
            eprintln!("Response sent successfully");
            Ok(())
        }

        pub fn recv_work_msg(&mut self) -> Result<ServerWorkPacket, anyhow::Error> {
            eprintln!("Starting to receive server work message...");
            
            // Read message size
            let mut sz_buf = [0; 8];
            match self.stream.recv_msg_chunk(&mut sz_buf) {
                Ok(_) => eprintln!("Successfully read size header: {:?}", sz_buf),
                Err(e) => {
                    eprintln!("Failed to read size header: {:?}", e);
                    return Err(e.into());
                }
            }
            
            let sz = u64::from_be_bytes(sz_buf);
            eprintln!("Received size header: {} bytes", sz);
            
            // Validate message size
            const MAX_MESSAGE_SIZE: u64 = MSG_SIZE_BYTES as u64;
            if sz > MAX_MESSAGE_SIZE {
                return Err(anyhow::anyhow!(
                    "Message size too large: {} bytes (max: {})",
                    sz,
                    MAX_MESSAGE_SIZE
                ));
            }
            
            // Read the message
            let mut buf = vec![0; sz as usize];
            self.stream.recv_msg_chunk(&mut buf)?;
            eprintln!("Received message payload");
            
            // Deserialize the message
            let packet = ServerWorkPacket::from_bytes(&buf)?;
            eprintln!("Successfully deserialized server work packet");
            
            Ok(packet)
        }
    }
}
