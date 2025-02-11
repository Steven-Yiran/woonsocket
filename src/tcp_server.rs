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
    eprintln!("Server bound successfully, waiting for connections...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                eprintln!("New connection accepted from {:?}", stream.peer_addr());
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
    
    eprintln!("[{}] Waiting to receive work packet...", peer_addr);
    let work_packet = match client_conn.recv_work_msg() {
        Ok(packet) => {
            eprintln!("[{}] Successfully received work packet", peer_addr);
            packet
        }
        Err(e) => {
            eprintln!("[{}] Failed to receive work packet: {:?}", peer_addr, e);
            return Err(e);
        }
    };
    
    eprintln!("[{}] Processing work packet...", peer_addr);
    let server_work_packet = work_packet.do_work();
    
    eprintln!("[{}] Sending response packet...", peer_addr);
    match server_conn.send_work_msg(server_work_packet) {
        Ok(_) => eprintln!("[{}] Successfully sent response", peer_addr),
        Err(e) => {
            eprintln!("[{}] Failed to send response: {:?}", peer_addr, e);
            return Err(e);
        }
    }
    
    eprintln!("[{}] Connection handled successfully", peer_addr);
    Ok(())
}
