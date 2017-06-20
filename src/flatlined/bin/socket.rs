use std::net::UdpSocket;
use flatconf::FlatConf;
use beat::*;
use log::*;
use std::net::IpAddr;

pub struct BeatListenSocket {
    socket: UdpSocket,
}

pub struct BeatSendSocket {
    socket: UdpSocket,
    conf: FlatConf,
}
    
pub type BeatResult = Result<(Beat, IpAddr), BeatError>;
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
        }
    }

    pub fn listen(&self) -> BeatResult {
        let mut buf = [0; 72];
        match self.socket.recv_from(&mut buf) {
            Ok((count, addr)) => {
                if count == 72 {
                    if log_enabled!(LogLevel::Debug) {
                        debug!("Beat received.");
                    }
                    Ok((Beat::from_bytes(buf), addr.ip()))
                } else {
                    Err(BeatError::WrongSize)
                }
            },
            Err(_) => Err(BeatError::ListenError),
        }
    }
}

impl BeatSendSocket {
    pub fn new(conf: &FlatConf) -> BeatSendSocket {
        BeatSendSocket {
            socket: UdpSocket::bind(("127.0.0.1", conf.port)).unwrap(),
            conf: conf.clone(),
        }
    }

    pub fn send_all(&self) -> () {
        let k = self.conf.clone();
        for s in k.server.unwrap() {
            match self.send(s.key.clone(), s.address.clone(), s.port) {
                Ok(_) => (),
                Err(_) => error!("Send error!"),
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
}
