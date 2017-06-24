extern crate toml;

use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::error::Error;
use std::process;
use server::Server;

#[derive(Debug, Deserialize, Clone)]
pub struct FlatConf {
    pub port: u16,
    pub logfile: String,
    pub socket: Option<String>,
    pub key: String,
    pub verbose: bool,
    pub server: Option<Vec<Server>>,
}

pub type ParsingResult = Result<FlatConf, String>;

impl FlatConf {
    pub fn parse(conf: &mut String) -> ParsingResult {
        let opts: FlatConf;
        match toml::from_str(conf) {
            Ok(conf) => {
                opts = conf;
                Ok(opts)
            }
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn parse_file(path: String) -> ParsingResult {
        let mut f;
        match File::open(Path::new(&path)) {
            Ok(open) => f = open,
            Err(_) => {
                error!("File not found: {}", path);
                process::exit(1);
            }
        }

        let mut buffer = String::new();

        if let Err(e) = f.read_to_string(&mut buffer) {
            panic!("Not able to read: {}", e.description());
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
    let conf = FlatConf::parse(&mut all).unwrap();
    let servers: Vec<Server>;
    let nopts = conf.clone();

    match nopts.server {
        Some(x) => servers = x.clone(),
        None => servers = Vec::new(),
    }

    assert_eq!(conf.port, 1337);
    assert_eq!(conf.logfile, "flat.log");
    assert_eq!(conf.socket.unwrap(), "flat.sock");
    assert_eq!(conf.key, "secret");
    assert_eq!(conf.verbose, true);
    assert_eq!(servers.len(), 2);
    assert_eq!(servers[0].address, "10.0.0.1");
    assert_eq!(servers[0].port, 8888);
    assert_eq!(servers[0].key, "foo");
    assert_eq!(servers[1].address, "10.0.0.2");
    assert_eq!(servers[1].port, 9999);
    assert_eq!(servers[1].key, "bar");
}

#[test]
fn partial_conf_parse_test() {
    let mut input = "port = 1337 \n
        logfile = 'flat.log' \n \
        socket = 'flat.sock' \n \
        key = 'secret' \n \
        verbose = true \n"
        .to_string();

    let conf = FlatConf::parse(&mut input).unwrap();

    assert_eq!(conf.port, 1337);
    assert_eq!(conf.logfile, "flat.log");
    assert_eq!(conf.socket.unwrap(), "flat.sock");
    assert_eq!(conf.key, "secret");
    assert_eq!(conf.verbose, true);
    assert_eq!(conf.server.is_none(), true);
}

#[test]
fn invalid_conf_parse_test() {
    let mut input = "part = 1337 \n \
        lögfile = 'flat.log' \n \
        sockt = 'flat.sock' \n \
        key = 'secret' \n \
        verböse = true \n"
        .to_string();

    assert!(FlatConf::parse(&mut input).is_err());
}
