use std::net::UdpSocket;
use flatconf::FlatConf;
use beat::Beat;
use log::*;

pub struct BeatListenSocket {
    socket: UdpSocket,
    conf: FlatConf,
}

pub struct BeatSendSocket {
    socket: UdpSocket,
    conf: FlatConf,
}

#[derive(Debug)]
pub enum BeatError {
    WrongSize,
    ListenError,
    SendError,
}
    
pub type BeatResult = Result<Beat, BeatError>;
pub type BeatSendResult = Result<(), BeatError>;


impl BeatListenSocket {
    fn bind(port: u16) -> UdpSocket {
        match UdpSocket::bind(("127.0.0.1", port)) {
            Ok(socket) => {
                if log_enabled!(LogLevel::Debug) {
                    debug!("Socket created on 127.0.0.1");
                }
                socket
            }
            Err(socket) => panic!("Couldn't bin to address: {}", socket),
        }
    }
    pub fn new(conf: &FlatConf) -> BeatListenSocket {
        BeatListenSocket {
            socket: BeatListenSocket::bind(conf.port),
            conf: conf.clone(),
        }
    }

    pub fn listen(&self) -> BeatResult {
        let mut buf = [0; 72];
        match self.socket.recv_from(&mut buf) {
            Ok((count, src)) => {
                if count == 72 {
                    if log_enabled!(LogLevel::Debug) {
                        debug!("Beat received.");
                    }
                    return Ok(Beat::from_bytes(buf));
                } else {
                    return Err(BeatError::WrongSize);
                }
            },
            Err(_) => Err(BeatError::ListenError),
        }
    }

    pub fn close(self) -> () {
        drop(self.socket);
    }
}

impl BeatSendSocket {
    pub fn new(conf: &FlatConf) -> BeatSendSocket {
        BeatSendSocket {
            socket: UdpSocket::bind("0.0.0.0").unwrap(),
            conf: conf.clone(),
        }
    }

    pub fn send_all(&self) -> () {
        let k = self.conf.clone();
        for s in k.server.into_iter() {
            match self.send(s.key.clone(), s.address.clone(), s.port) {
                Ok(_) => (),
                Err(err) => error!("Send error!"),
            }
        }
    }

    fn send(&self, key: String, addr: String, port: u16) -> BeatSendResult {
        match self.socket.send_to(&Beat::new(key.as_str()).into_bytes(), (addr.as_str(), port)) {
                Ok(send) => {
                    if log_enabled!(LogLevel::Debug) {
                        debug!("Send {} bytes!", send);
                    }
                    Ok(())
                },
                Err(_) => Err(BeatError::SendError),
            }
    }

    pub fn close(self) -> () {
        drop(self.socket);
    }
}