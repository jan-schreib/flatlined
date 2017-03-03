extern crate nanomsg;

use std::str;
use std::io::{Read, Write};
use log::LogLevel;

#[derive(PartialEq, Eq, Debug)]
enum IPCType {
    Bind,
    Connect,
}

pub struct IPC {
    socket: nanomsg::Socket,
    endpoint: nanomsg::Endpoint,
    form: IPCType,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum IPCMsg {
    Status = 1,
    Quit,
    Ok,
    None,
}

fn to_val(msg: &IPCMsg) -> u8 {
    *msg as u8
}

impl IPC {
    fn create_socket() -> nanomsg::Socket {
        match nanomsg::Socket::new(nanomsg::Protocol::Pair) {
            Ok(socket) => socket,
            Err(err) => panic!("{}", err),
        }
    }

    fn bind_endpoint(sock: &mut nanomsg::Socket, dest: &str) -> nanomsg::Endpoint {
        match sock.bind(dest) {
            Ok(s) => s,
            Err(err) => panic!("Failed to bind socket: {}", err),
        }
    }


    fn connect_endpoint(sock: &mut nanomsg::Socket, dest: &str) -> nanomsg::Endpoint {
        match sock.connect(dest) {
            Ok(ep) => ep,
            Err(err) => panic!("Failed to connect socket: {}", err),
        }
    }

    pub fn new_bind(sock: &str) -> IPC {
        let mut s = IPC::create_socket();
        let e = IPC::bind_endpoint(&mut s, sock);

        IPC {
            socket: s,
            endpoint: e,
            form: IPCType::Bind,
        }
    }

    pub fn new_connect(sock: &str) -> IPC {
        let mut s = IPC::create_socket();
        let e = IPC::connect_endpoint(&mut s, sock);

        IPC {
            socket: s,
            endpoint: e,
            form: IPCType::Connect,
        }
    }

    pub fn shutdown(&mut self) -> () {
        self.endpoint.shutdown().unwrap();
    }

    pub fn receive_msg(&mut self) -> IPCMsg {
        let mut buffer = [0u8; 1];
        match self.socket.read(&mut buffer) {
            Ok(count) => {
                if log_enabled!(LogLevel::Debug) {
                    debug!("Read {} bytes! Value {}", count, buffer[0]);
                }
            }
            Err(err) => error!("Problem while reading: {}", err),
        }
        match buffer[0] {
            1 => IPCMsg::Status,
            2 => IPCMsg::Quit,
            3 => IPCMsg::Ok,
            _ => IPCMsg::None,
        }
    }

    pub fn send_msg(&mut self, msg: IPCMsg) -> () {
        if self.form == IPCType::Bind {
            error!("Called send_msg on a bound socket. Use a connected socket!")
        }
        match self.socket.write(&[to_val(&msg)]) {
            Ok(_) => {
                if log_enabled!(LogLevel::Debug) {
                    debug!("Message sent ! {} ", to_val(&msg));
                }
            }
            Err(err) => error!("Problem while writing: {}", err),
        };
    }
}

#[test]
fn socket_bind_test() {
    let mut ipc = IPC::new_bind("ipc:///tmp/ipc-bind-test.ipc");
    assert_eq!(ipc.form, IPCType::Bind);

    ipc.shutdown();
}

#[test]
fn socket_connect_test() {
    let mut ipc = IPC::new_bind("ipc:///tmp/ipc-connect-test.ipc");
    let mut ipc2 = IPC::new_connect("ipc:///tmp/ipc-connect-test.ipc");
    assert_eq!(ipc2.form, IPCType::Connect);

    ipc.shutdown();
    ipc2.shutdown();
}

#[test]
fn msg_test() {
    let mut ipc = IPC::new_bind("ipc:///tmp/ipc-msg-test.ipc");
    let mut ipc2 = IPC::new_connect("ipc:///tmp/ipc-msg-test.ipc");

    ipc2.send_msg(IPCMsg::Ok);
    ipc2.send_msg(IPCMsg::None);
    ipc2.send_msg(IPCMsg::Status);
    ipc2.send_msg(IPCMsg::Quit);

    let mut incoming = ipc.receive_msg();
    assert_eq!(incoming, IPCMsg::Ok);
    incoming = ipc.receive_msg();
    assert_eq!(incoming, IPCMsg::None);
    incoming = ipc.receive_msg();
    assert_eq!(incoming, IPCMsg::Status);
    incoming = ipc.receive_msg();
    assert_eq!(incoming, IPCMsg::Quit);

    ipc.shutdown();
    ipc2.shutdown();
}

#[test]
#[should_panic]
fn socket_bind_panic_test() {
    IPC::new_bind("broken");
}

#[test]
#[should_panic]
fn socket_connect_panic_test() {
    IPC::new_connect("broken");
}
