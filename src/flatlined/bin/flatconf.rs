extern crate toml;

use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::error::Error;
use std::process;

#[derive(Debug, Deserialize)]
pub struct Server {
    address: String,
    port: u16,
    key: String,
}

#[derive(Debug, Deserialize)]
pub struct FlatConf {
    port: u16,
    logfile: String,
    socket: String,
    key: String,
    verbose: bool,
    server: Vec<Server>,
}

impl FlatConf {
    pub fn parse(conf: &mut String) -> FlatConf {
        let opts: FlatConf = toml::from_str(conf).unwrap();
        opts
    }

    pub fn parse_file(path: String) -> FlatConf {
        let mut f;
        match File::open(Path::new(&path)) {
            Ok(open) => f = open,
            Err(_) => {
                error!("File not found: {}", path);
                process::exit(1);
            }
        }

        let mut buffer = String::new();

        match f.read_to_string(&mut buffer) {
            Err(e) => panic!("Not able to read: {}", e.description()),
            _ => {}
        }

        FlatConf::parse(&mut buffer)
    }
}

#[test]
fn parse_test() {
    let input = "port = 1337 \n logfile = 'flat.log' \n socket = 'flat.sock' \n key = 'secret' \n \
                 verbose = true \n"
        .to_string();
    let input2 = "[[server]] \n address = '10.0.0.1' \n port = 8888 \n key = 'foo' \n [[server]] \
                  \n address = '10.0.0.2' \n port = 9999 \n key = 'bar' \n";
    let mut all = input.clone() + input2;
    let conf = FlatConf::parse(&mut all);

    assert_eq!(conf.port, 1337);
    assert_eq!(conf.logfile, "flat.log");
    assert_eq!(conf.socket, "flat.sock");
    assert_eq!(conf.key, "secret");
    assert_eq!(conf.verbose, true);
    assert_eq!(conf.server.len(), 2);
    assert_eq!(conf.server[0].address, "10.0.0.1");
    assert_eq!(conf.server[0].port, 8888);
    assert_eq!(conf.server[0].key, "foo");
    assert_eq!(conf.server[1].address, "10.0.0.2");
    assert_eq!(conf.server[1].port, 9999);
    assert_eq!(conf.server[1].key, "bar");
}
