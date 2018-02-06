use std::fmt;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Server {
    pub address: String,
    pub port: u16,
    pub key: String,
}

impl fmt::Display for Server {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        try!(fmt.write_str(&self.address));
        try!(fmt.write_str(":"));
        try!(fmt.write_str(&self.port.to_string()));
        Ok(())
    }
}
