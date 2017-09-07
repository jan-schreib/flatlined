use server::Server;
use std::fmt;

#[derive(Debug, Deserialize, Clone)]
pub struct Statistic {
    pub send_beats: u64,
    pub recv_beats: u64,
    pub server: Server,
}

impl fmt::Display for Statistic {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
	try!(fmt.write_str("Tx: "));
        try!(fmt.write_str(&self.send_beats.to_string()));
        try!(fmt.write_str(" Rx: "));
        try!(fmt.write_str(&self.recv_beats.to_string()));
        try!(fmt.write_str(" Host: "));
        try!(fmt.write_str(&self.server.to_string()));
        Ok(())
    }
}

impl Statistic {
    pub fn new(s: &Server) -> Statistic {
        Statistic {
            send_beats: 0,
            recv_beats: 0,
            server: s.clone(),
        }
    }

    pub fn incr_send(&mut self) -> () {
        self.send_beats += 1;
    }

    pub fn incr_recv(&mut self) -> () {
        self.recv_beats += 1;
    }
}
