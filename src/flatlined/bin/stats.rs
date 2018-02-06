use server::Server;
use std::fmt;
use std::time::*;

#[derive(Debug, Deserialize, Clone)]
pub struct Statistic {
    pub send_beats: u64,
    pub recv_beats: u64,
    pub server: Server,
    pub timestamp: u64
}

impl fmt::Display for Statistic {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        try!(fmt.write_str("Tx: "));
        try!(fmt.write_str(&self.send_beats.to_string()));
        try!(fmt.write_str(" Rx: "));
        try!(fmt.write_str(&self.recv_beats.to_string()));
        try!(fmt.write_str(" Host: "));
        try!(fmt.write_str(&self.server.to_string()));
        if self.timestamp != 0 && SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() - self.timestamp > 60 {
            try!(fmt.write_str(" OFFLINE: "));
        } else {
            try!(fmt.write_str(" ONLINE"));
        }
        try!(fmt.write_str("\n"));
        Ok(())
    }
}

impl Statistic {
    pub fn new(s: &Server) -> Statistic {
        Statistic {
            send_beats: 0,
            recv_beats: 0,
            server: s.clone(),
            timestamp: 0,
        }
    }

    pub fn incr_send(&mut self) {
        self.send_beats += 1;
    }

    pub fn incr_recv(&mut self) {
        self.recv_beats += 1;
    }

    pub fn set_timestamp(&mut self, timestamp: u64) {
        self.timestamp = timestamp;
    }
}
