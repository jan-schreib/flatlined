#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate log;

extern crate env_logger;
extern crate nix;
extern crate daemonize;
extern crate clap;

mod flatconf;
mod ipc;

use ipc::*;
use flatconf::FlatConf;
use clap::{Arg, App};
use daemonize::Daemonize;
use nix::unistd;
use std::process;
use std::{thread, time};

static DEFAULT_CONF: &'static str = "/etc/flat.conf";
static PIDFILE: &'static str = "/var/run/flatlined.pid";
static FLATUSER: &'static str = "_flatlined";
static FLATGROUP: &'static str = "_flatlined";
static FLATSOCK: &'static str = "ipc:///var/run/flatlined.sock";

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
            .user(FLATUSER)
            .group(FLATGROUP)
            .privileged_action(|| "Executed before drop privileges");

        match daemonize.start() {
            Ok(_) => info!("Success, daemonized"),
            Err(e) => error!("{}", e),
        }
    }


    let mut ipc = IPC::new_bind(FLATSOCK);
    thread::spawn(move || loop {
        match ipc.receive_msg() {
            IPCMsg::Status => println!("Status msg received."),
            _ => println!("Unknown msg received."),
        }
    });

    thread::sleep(time::Duration::from_millis(60000));
}