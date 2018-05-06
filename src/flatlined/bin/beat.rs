extern crate quickcheck;

use std::time::*;
use blake2_rfc::blake2b::Blake2b;
use constant_time_eq::constant_time_eq;
use std::str;

static DEFAULT_MSG: &'static str = "beat";

pub struct Beat {
    pub timestamp: u64,
    hash: [u8; 64],
}

#[derive(Debug)]
pub enum BeatError {
    WrongSize,
    ListenError,
    SendError,
    WrongChecksum,
}

impl PartialEq for Beat {
    fn eq(&self, other: &Beat) -> bool {
        self.timestamp == other.timestamp && constant_time_eq(&self.hash, &other.hash)
    }
}

impl Clone for Beat {
    fn clone(&self) -> Beat {
        let mut b = Beat {
            timestamp: self.timestamp,
            hash: [0; 64],
        };
        b.hash.clone_from_slice(&self.hash);
        b
    }
}

fn u64_to_u8arr(value: u64) -> [u8; 8] {
    let mut ret: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    ret[0] |= (value & 0xFF) as u8;
    ret[1] |= ((value & (0xFF << 8)) >> 8) as u8;
    ret[2] |= ((value & (0xFF << 16)) >> 16) as u8;
    ret[3] |= ((value & (0xFF << 24)) >> 24) as u8;
    ret[4] |= ((value & (0xFF << 32)) >> 32) as u8;
    ret[5] |= ((value & (0xFF << 40)) >> 40) as u8;
    ret[6] |= ((value & (0xFF << 48)) >> 48) as u8;
    ret[7] |= ((value & (0xFF << 56)) >> 56) as u8;
    ret
}

fn u8arr_to_u64(value: [u8; 8]) -> u64 {
    let mut ret = (value[7] as u64) << 56;
    ret |= (value[6] as u64) << 48;
    ret |= (value[5] as u64) << 40;
    ret |= (value[4] as u64) << 32;
    ret |= (value[3] as u64) << 24;
    ret |= (value[2] as u64) << 16;
    ret |= (value[1] as u64) << 8;
    ret | value[0] as u64
}

impl Beat {
    fn create_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn create_checksum(key: &str, time: &[u8]) -> [u8; 64] {
        let mut ctx = Blake2b::with_key(64, key.as_bytes());
        ctx.update(DEFAULT_MSG.as_bytes());
        ctx.update(time);
        let temp = ctx.finalize();
        let res = temp.as_bytes();
        let mut ret: [u8; 64] = [0; 64];
        for (i, v) in res.iter().enumerate() {
            ret[i] = *v;
        }
        ret
    }

    pub fn new(server_key: &str) -> Beat {
        let time = Beat::create_timestamp();
        let key = if server_key.is_empty() || server_key.len() >= 64 {
            ""
        } else {
            server_key
        };

        let hash = Beat::create_checksum(key, &(u64_to_u8arr(time)));
        Beat {
            timestamp: time,
            hash: hash,
        }
    }

    pub fn from_bytes(data: [u8; 72]) -> Beat {
        let mut ts = [0u8; 8];
        let mut cs = [0u8; 64];

        ts[..].clone_from_slice(&data[..8]);
        cs[..].clone_from_slice(&data[8..]);

        Beat {
            timestamp: u8arr_to_u64(ts),
            hash: cs,
        }
    }

    pub fn into_bytes(self) -> [u8; 72] {
        let mut ret = [0u8; 72];
        let ts = u64_to_u8arr(self.timestamp);

        ret[..8].clone_from_slice(&ts[..8]);
        ret[8..].clone_from_slice(&self.hash[..64]);

        ret
    }

    pub fn verify_beat(&self, key: &str) -> Result<bool, BeatError> {
        let sum = Beat::create_checksum(key, &u64_to_u8arr(self.timestamp));
        if constant_time_eq(&sum, &self.hash) {
            Ok(true)
        } else {
            Err(BeatError::WrongChecksum)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use beat::quickcheck::quickcheck;
    use core::u64;

    #[test]
    fn new_qc_test() {
        fn qc(input: Vec<u8>) -> bool {
            let msg = Beat::new(str::from_utf8(&input).unwrap());
            let bmsg = msg.clone().into_bytes();
            let nbmsg = Beat::from_bytes(bmsg);

            nbmsg == msg
        }
        quickcheck(qc as fn(Vec<u8>) -> bool);
    }

    #[test]
    fn verification_test() {
        let b = Beat::new("key");
        assert!(b.verify_beat("key").unwrap(), true);
    }

    #[test]
    fn verification_fails_test() {
        let b = Beat::new("key");
        assert!(b.verify_beat("not_the_key").is_err());
    }

    #[test]
    fn to_bytes_test() {
        let msg = Beat::new("foo");
        let bmsg = msg.clone().into_bytes();
        let nbmsg = Beat::from_bytes(bmsg);

        assert!(nbmsg == msg, true);
    }

    #[test]
    fn from_bytes_test() {
        let ts = u64_to_u8arr(u64::max_value());
        let hs = Beat::create_checksum("foo", &ts);
        let mut data = [0u8; 72];

        for b in 0..8 {
            data[b] = ts[b];
        }

        for bb in 0..64 {
            data[bb + 8] = hs[bb];
        }

        let beat = Beat::from_bytes(data);
        let bbeat = Beat {
            timestamp: u64::max_value(),
            hash: Beat::create_checksum("foo", &ts),
        };

        assert!(beat == bbeat, true);

    }

    #[test]
    fn beat_eq_test() {
        let a = Beat::new("foo");
        let b = Beat::new("foo");
        assert!(a.timestamp == b.timestamp);
        assert!(a == b, true);
    }

    #[test]
    fn beat_ne_test() {
        let a = Beat {
            timestamp: 1u64,
            hash: Beat::create_checksum("foo", &u64_to_u8arr(1u64)),
        };
        let b = Beat::new("foo");
        let c = Beat::new("bar");
        assert!(a != b, true);
        assert!(a != c, true);
    }

    #[test]
    fn u64to8arrtou64_test() {
        let big = u64::max_value();
        let sml = u64_to_u8arr(big);
        let nbi = u8arr_to_u64(sml);

        let min = u64::min_value();
        let mml = u64_to_u8arr(min);
        let nmi = u8arr_to_u64(mml);

        assert_eq!(big, nbi);
        assert_eq!(min, nmi);
    }
}
