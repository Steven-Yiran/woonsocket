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

        // TODO: Students can add their own members,
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
            // TODO: Students should implement this method.
            // serialize.rs contains how ClientWorkPacket is serialized. The
            // resulting bytes are variable length but guaranteed to be <
            // MSG_SIZE_BYTES so students should account for this when sending a
            // ClientWorkPacket.
            
            let mut buf = vec![0; MSG_SIZE_BYTES];
            let sz = work_packet.to_bytes(&mut buf)?; //sz is u64
            // send size of message then message
            self.stream.send_msg_chunk(&sz.to_be_bytes())?;
            self.stream.send_msg_chunk(&buf[..sz as usize])?;
            Ok(())
        }

        pub fn recv_work_msg(&mut self) -> Result<ClientWorkPacket, anyhow::Error> {
            // TODO: Students should implement this method
            let mut sz_buf = [0; 8];
            self.stream.recv_msg_chunk(&mut sz_buf)?;
            let sz = u64::from_le_bytes(sz_buf);
            let mut buf = vec![0; sz as usize];
            self.stream.recv_msg_chunk(&mut buf)?;
            let packet = ClientWorkPacket::from_bytes(&buf)?;
            Ok(packet)
        }

        // TODO: Students can implement their own methods
    }

	// TODO: Students can implement their own helpers
}

pub mod work_response {
    use super::*;

    pub struct ServerWorkPacketConn {
        stream: ChunkedTcpStream,

        // TODO: Students can add their own members,
    }

    impl ServerWorkPacketConn {
        pub fn new(stream: &TcpStream) -> Self {
            let stream = stream.try_clone().expect("Failed to clone stream");
            let chunked_stream = ChunkedTcpStream::new(stream);
            Self { stream: chunked_stream }
        }

        pub fn send_work_msg(&mut self, packet: ServerWorkPacket) -> Result<(), anyhow::Error> {
            // TODO: Students should implement this method.
            // serialize.rs contains how ServerWorkPacket is serialized. The
            // resulting bytes are variable length and can be larger than
            // MSG_SIZE_BYTES so students should account for this when sending
            // and ServerWorkPacket.
            //
            // NOTE: for Project-0. We can assume that ServerWorkPacket will
            // always be < MSG_SIZE_BYTES. This will change in the next projects
            
            let mut buf = vec![0; MSG_SIZE_BYTES];
            let sz = packet.to_bytes(&mut buf)?; //sz is u64
            // send size of message then message
            self.stream.send_msg_chunk(&sz.to_be_bytes())?;
            self.stream.send_msg_chunk(&buf[..sz as usize])?;
            Ok(())
        }

        pub fn recv_work_msg(&mut self) -> Result<ServerWorkPacket, anyhow::Error> {
            // TODO: Students should implement this method
            eprintln!("Starting to receive work message...");
            let mut sz_buf = [0; 8];
            match self.stream.recv_msg_chunk(&mut sz_buf) {
                Ok(_) => eprintln!("Successfully received size buffer: {:?}", sz_buf),
                Err(e) => {
                    eprintln!("Failed to receive size buffer: {:?}", e);
                    return Err(e.into());
                }
            }
            
            let sz = u64::from_le_bytes(sz_buf);
            eprintln!("Decoded message size: {} bytes", sz);
            
            let mut buf = vec![0; sz as usize];
            match self.stream.recv_msg_chunk(&mut buf) {
                Ok(_) => eprintln!("Successfully received message buffer of size {}", sz),
                Err(e) => {
                    eprintln!("Failed to receive message buffer: {:?}", e);
                    return Err(e.into());
                }
            }
            
            match ServerWorkPacket::from_bytes(&buf) {
                Ok(packet) => {
                    eprintln!("Successfully deserialized ServerWorkPacket");
                    Ok(packet)
                }
                Err(e) => {
                    eprintln!("Failed to deserialize ServerWorkPacket: {:?}", e);
                    Err(e)
                }
            }
        }

        // TODO: Students can implement their own methods
    }
	// TODO: Students can implement their own helpers
}
