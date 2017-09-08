use std::net::UdpSocket;
use flatconf::FlatConf;
use beat::*;
use log::*;
use std::error::Error;
use std::net::*;
use std::str::FromStr;
use trust_dns_resolver::Resolver;
use trust_dns_resolver::config::*;

pub struct BeatListenSocket {
    socket: UdpSocket,
}

pub struct BeatSendSocket {
    socket: UdpSocket,
    pub conf: FlatConf,
}

pub type BeatResult = Result<(Beat, IpAddr), BeatError>;
pub type BeatSendResult = Result<(), BeatError>;

impl BeatListenSocket {
    fn bind(port: u16) -> UdpSocket {
        match UdpSocket::bind(("0.0.0.0", port)) {
            Ok(socket) => {
                if log_enabled!(LogLevel::Debug) {
                    debug!("Socket created on 0.0.0.0");
                }
                socket
            }
            Err(socket) => panic!("Couldn't bin to address: {}", socket),
        }
    }
    pub fn new(conf: &FlatConf) -> BeatListenSocket {
        BeatListenSocket { socket: BeatListenSocket::bind(conf.port) }
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
            }
            Err(_) => Err(BeatError::ListenError),
        }
    }
}

impl BeatSendSocket {
    pub fn new(conf: &FlatConf) -> BeatSendSocket {
        BeatSendSocket {
            socket: UdpSocket::bind(("0.0.0.0", conf.port)).unwrap(),
            conf: conf.clone(),
        }
    }

    fn get_ip(hostname: &str) -> Result<IpAddr, String> {
        #[cfg(target_family="unix")]
        let mut resolver = Resolver::from_system_conf().unwrap();
        #[cfg(target_family="windows")]
        let mut resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default())
            .unwrap();

        let response = resolver.lookup_ip(hostname).unwrap();

        match response.iter().next() {
            Some(ip) => return Ok(ip.clone()),
            _ => return Err(hostname.to_owned()),
        }
    }

    pub fn send(&self, key: String, addr: String, port: u16) -> BeatSendResult {
        let msg = Beat::new(key.as_str()).into_bytes();
        match BeatSendSocket::get_ip(&addr) {
            Ok(ip) => {
                match self.socket.send_to(&msg, (ip, port)) {
                    Ok(send) => {
                        if log_enabled!(LogLevel::Debug) {
                            debug!("Send {} bytes!", send);
                        }
                        Ok(())
                    }
                    Err(e) => {
                        println!("{}", e.description());
                        Err(BeatError::SendError)
                    }
                }
            }
            Err(e) => {
                panic!(e);
            }
        }
    }
}

#[test]
#[ignore]
fn test_get_ip_from_hostname() {
    let ip = BeatSendSocket::get_ip("www.heise.de").unwrap();
    if ip.is_ipv4() {
        assert_eq!(IpAddr::from_str("193.99.144.80").unwrap(), ip);
    } else {
        assert_eq!(
            IpAddr::from_str("2a02:2e0:3fe:1001:7777:772e:2:85").unwrap(),
            ip
        );
    }
}

#[test]
#[ignore]
fn test_get_ip_from_ip() {
    match BeatSendSocket::get_ip("193.99.144.80") {
        Ok(_) => assert!(true),
        Err(e) => assert_eq!("193.99.144.80", e),
    }
}
