extern crate nanomsg;

#[macro_use]
extern crate log;

use std::str;
use std::io::{Read, Write};
use log::LogLevel;

pub struct IPC {
    socket: nanomsg::Socket,
    endpoint: nanomsg::Endpoint,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum IPCMsgType {
    Status = 1,
    Quit,
    Ok,
    Error,
    Any,
}

pub struct IPCMsg {
    pub typ: IPCMsgType,
    pub msg: [u8; 1024],
}

impl IPCMsg {
    pub fn create_payload(&mut self, msg: &str) -> () {
        let mut ret = [0u8; 1024];
        let mut i = 0;
        for c in msg.as_bytes() {
            ret[i] = *c;
            i += 1;
        }
        self.msg = ret;
    }
}

fn to_val(msg: &IPCMsgType) -> u8 {
    *msg as u8
}

pub type IPCRecvResult = Result<IPCMsg, std::io::Error>;
pub type IPCSendResult = Result<(), std::io::Error>;

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
        }
    }

    pub fn new_connect(sock: &str) -> IPC {
        let mut s = IPC::create_socket();
        let e = IPC::connect_endpoint(&mut s, sock);

        IPC {
            socket: s,
            endpoint: e,
        }
    }

    pub fn shutdown(&mut self) -> () {
        self.endpoint.shutdown().unwrap();
    }

    pub fn set_send_timeout(&mut self, time: isize) -> () {
        self.socket.set_send_timeout(time).unwrap();
    }

    pub fn set_recv_timeout(&mut self, time: isize) -> () {
        self.socket.set_receive_timeout(time).unwrap();
    }

    pub fn receive_msg(&mut self) -> IPCRecvResult {
        let mut buffer = [0u8; 1025];
        let mut ret = IPCMsg {
            typ: IPCMsgType::Any,
            msg: [0u8; 1024],
        };
        match self.socket.read(&mut buffer) {
            Ok(count) => {
                if log_enabled!(LogLevel::Debug) {
                    debug!("Read {} bytes!", count);
                }
                match buffer[0] {
                    1 => ret.typ = IPCMsgType::Status,
                    2 => ret.typ = IPCMsgType::Quit,
                    3 => ret.typ = IPCMsgType::Ok,
                    _ => ret.typ = IPCMsgType::Any,
                };
                ret.msg.clone_from_slice(&buffer[1..1025]);
                Ok(ret)
            }
            Err(err) => Err(err),
        }
    }

    pub fn send_msg(&mut self, msg: IPCMsg) -> IPCSendResult {
        let mut buffer = [0u8; 1025];
        buffer[0] = to_val(&msg.typ);
        buffer[1..].clone_from_slice(&msg.msg);
        match self.socket.write(&buffer) {
            Ok(count) => {
                if log_enabled!(LogLevel::Debug) {
                    debug!("Message: {} of size: {} send",
                           std::str::from_utf8(&msg.msg).unwrap(),
                           count);
                }
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}

#[test]
fn msg_test() {
    let mut ipc = IPC::new_bind("ipc:///tmp/ipc-msg-test.ipc");
    let mut ipc2 = IPC::new_connect("ipc:///tmp/ipc-msg-test.ipc");
    let mut msg = IPCMsg {
        typ: IPCMsgType::Status,
        msg: [0u8; 1024],
    };
    msg.create_payload("test payload");
    ipc2.send_msg(msg).unwrap();
    let incoming = ipc.receive_msg().unwrap();

    assert_eq!(incoming.typ, IPCMsgType::Status);
    assert_eq!(std::str::from_utf8(&incoming.msg[..12]).unwrap(),
               "test payload");

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
