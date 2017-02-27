#[macro_use]
extern crate serde_derive;

extern crate clap;

mod flatconf;

use flatconf::FlatConf;
use clap::{Arg, App};
use std::io::Write;

static DEFAULT_CONF: &'static str = "flat.conf";

fn main () {
    let matches = App::new("flatlined - a heartbeat server")
        .version("0.1")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Sets a custom config file")
            .takes_value(true))
        .get_matches();

    let opts: FlatConf;
    
    match matches.value_of("config") {
        Some(x) => opts =  FlatConf::parse_file(x.to_owned()),
        None => opts = FlatConf::parse_file(DEFAULT_CONF.to_owned()),
    }
}
