use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub const MSG_SIZE_BYTES: usize = 128;

pub struct ChunkedTcpStream(TcpStream);

impl ChunkedTcpStream {
    pub fn send_msg_chunk(&mut self, bytes: &[u8]) -> Result<(), anyhow::Error> {
        assert!(bytes.len() <= MSG_SIZE_BYTES);
        self.0.write_all(&bytes)?;
        self.0.flush()?;
        Ok(())
    }

    pub fn recv_msg_chunk(&mut self, bytes: &mut [u8]) -> Result<(), anyhow::Error> {
        assert!(bytes.len() <= MSG_SIZE_BYTES);
        self.0.read_exact(bytes)?;
        Ok(())
    }

    pub fn new(tcp: TcpStream) -> Self {
        Self(tcp)
    }
}
