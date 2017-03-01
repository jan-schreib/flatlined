#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate log;

extern crate env_logger;
extern crate nix;
extern crate daemonize;
extern crate clap;

mod flatconf;

use flatconf::FlatConf;
use clap::{Arg, App};
use daemonize::Daemonize;
use nix::unistd;
use std::process;

static DEFAULT_CONF: &'static str = "/etc/flat.conf";
static PIDFILE: &'static str = "/var/run/flatlined.pid";

fn uidcheck() -> () {
    if unistd::geteuid() != 0 {
        error!("Starting this application requires root privileges");
        process::exit(1);
    } else {
        return;
    }
}

fn main() {
    env_logger::init().unwrap();
    uidcheck();

    let matches = App::new("flatlined - a heartbeat server")
        .version("0.1")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Sets a custom config file")
            .takes_value(true))
        .arg(Arg::with_name("debug")
            .short("d")
            .long("debug")
            .help("Debug mode, don't detach or become a daemon.")
            .takes_value(false))
        .get_matches();

    let opts: FlatConf;

    match matches.value_of("config") {
        Some(x) => opts = FlatConf::parse_file(x.to_owned()),
        None => opts = FlatConf::parse_file(DEFAULT_CONF.to_owned()),
    }

    if !matches.is_present("debug") {
        let daemonize = Daemonize::new()
            .pid_file(PIDFILE)
            .chown_pid_file(false)
            .working_directory("/tmp")
            .user("_flatlined")
            .group("_flatlined")
            .privileged_action(|| "Executed before drop privileges");

        match daemonize.start() {
            Ok(_) => info!("Success, daemonized"),
            Err(e) => error!("{}", e),
        }
    }

    loop {}
}
