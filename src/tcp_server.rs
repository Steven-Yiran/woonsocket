use crate::{
    protocol::{work_request::ClientWorkPacketConn, work_response::ServerWorkPacketConn},
    serialize::ClientWorkPacket,
};

use std::{
    net::{SocketAddrV4, TcpListener, TcpStream},
    thread,
};

pub fn tcp_server(addr: SocketAddrV4) -> Result<(), anyhow::Error> {
    eprintln!("Starting TCP server on {:?}", addr);
    let listener = TcpListener::bind(addr).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    if let Err(e) = handle_conn(stream) {
                        eprintln!("Connection handler error: {:?}", e);
                    }
                    eprintln!("Connection handler thread terminated");
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {:?}", e);
            }
        }
    }

    Ok(())
}

fn handle_conn(stream: TcpStream) -> Result<(), anyhow::Error> {
    let peer_addr = stream.peer_addr()?;
    eprintln!("Starting to handle connection from {:?}", peer_addr);
    
    let mut client_conn = ClientWorkPacketConn::new(&stream);
    let mut server_conn = ServerWorkPacketConn::new(&stream);
    
    let work_packet = match client_conn.recv_work_msg() {
        Ok(packet) => {
            packet
        }
        Err(e) => {
            return Err(e);
        }
    };
    
    let server_work_packet = work_packet.do_work();
    
    match server_conn.send_work_msg(server_work_packet) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("[{}] Failed to send response: {:?}", peer_addr, e);
            return Err(e);
        }
    }
    Ok(())
}
