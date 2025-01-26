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
        pub fn send_work_msg(
            &mut self,
            work_packet: ClientWorkPacket,
        ) -> Result<(), anyhow::Error> {
            // TODO: Students should implement this method.
            // serialize.rs contains how ClientWorkPacket is serialized. The
            // resulting bytes are variable length but guaranteed to be <
            // MSG_SIZE_BYTES so students should account for this when sending a
            // ClientWorkPacket.

		unimplemented!()
        }

        pub fn recv_work_msg(&mut self) -> Result<ClientWorkPacket, anyhow::Error> {
            // TODO: Students should implement this method
		unimplemented!()
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
        pub fn send_work_msg(&mut self, packet: ServerWorkPacket) -> Result<(), anyhow::Error> {
            // TODO: Students should implement this method.
            // serialize.rs contains how ServerWorkPacket is serialized. The
            // resulting bytes are variable length and can be larger than
            // MSG_SIZE_BYTES so students should account for this when sending
            // and ServerWorkPacket.
            //
            // NOTE: for Project-0. We can assume that ServerWorkPacket will
            // always be < MSG_SIZE_BYTES. This will change in the next projects
		unimplemented!()
        }

        pub fn recv_work_msg(&mut self) -> Result<ServerWorkPacket, anyhow::Error> {
            // TODO: Students should implement this method
		unimplemented!()
        }

        // TODO: Students can implement their own methods
    }
	// TODO: Students can implement their own helpers
}
