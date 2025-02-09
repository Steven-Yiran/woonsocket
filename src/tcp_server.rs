use crate::{
    protocol::{work_request::ClientWorkPacketConn, work_response::ServerWorkPacketConn},
    serialize::ClientWorkPacket,
};

use std::{
    net::{SocketAddrV4, TcpListener, TcpStream},
    thread,
};

pub fn tcp_server(addr: SocketAddrV4) {
    // TODO: Students will have to write this code.
    // There's nothing special here except the server should listen for new
    // client connections and then spin off a new thread to handle that
    // connection.
    
    let listener = TcpListener::bind(addr).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(move || {
            if let Err(e) = handle_conn(stream) {
                eprintln!("Error: {:?}", e);
            });
        }
    }
}

fn handle_conn(stream: TcpStream) -> Result<(), anyhow::Error> {
    // TODO: Students will have to write this code.
    // NOTE: It might be helpful to look at protocol.rs first. You'll probably
    // be implementing that alongside this function.
    //
    // This function handles one client connection. It receives a
    // ClientWorkPacket, does work using ClientWorkPacket::do_work which returns
    // a ServerWorkPacket, then sends this ServerWorkPacket back to the client.
    
    let mut client_conn = ClientWorkPacketConn::new(stream);
    let mut server_conn = ServerWorkPacketConn::new(stream);
    let work_packet = match client_conn.recv_work_msg() {
        Ok(packet) => packet,
        Err(e) => {
            eprintln!("Error receiving work packet: {:?}", e);
            return Err(e);
        }
    };
    let server_work_packet = work_packet.do_work();
    server_conn.send_work_msg(server_work_packet).unwrap();
}
