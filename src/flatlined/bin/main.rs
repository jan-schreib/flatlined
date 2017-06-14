#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate log;

extern crate blake2_rfc;
extern crate constant_time_eq;
extern crate env_logger;
extern crate nix;
extern crate daemonizer;
extern crate clap;
extern crate ipc;

mod flatconf;
mod beat;
mod socket;
mod server;
mod stats;

use ipc::*;
use socket::{BeatListenSocket, BeatSendSocket};
use flatconf::FlatConf;
use stats::Statistic;
use clap::{Arg, App};
use daemonizer::Daemonize;
use nix::unistd;
use std::process;
use std::thread;
use std::fs;
use std::os::unix::fs::PermissionsExt;

static DEFAULT_CONF: &'static str = "/etc/flat.conf";
static PIDFILE: &'static str = "/var/run/flatlined.pid";
static FLATUSER: &'static str = "_flatlined";
static FLATGROUP: &'static str = "_flatlined";
static FLATSOCK: &'static str = "ipc:///var/run/flatlined.sock";
static FLATSOCKPATH: &'static str = "/var/run/flatlined.sock";

fn uidcheck() -> () {
    if unistd::geteuid() != 0 {
        error!("Starting this application requires root privileges");
        process::exit(1);
    } else {
        return;
    }
}

fn ipc_handler(stats: Vec<Statistic>) -> () {
    let mut ipc = IPC::new_bind(FLATSOCK);

    let meta = fs::metadata(FLATSOCKPATH).unwrap();
    let mut permissions = meta.permissions();
    permissions.set_mode(0o666);
    fs::set_permissions(FLATSOCKPATH, permissions).unwrap();

    thread::spawn(move || loop {
                      let mut m = IPCMsg {
                          typ: IPCMsgType::Any,
                          msg: [0; 1024],
                      };
                      match ipc.receive_msg().unwrap().typ {
                          IPCMsgType::Ok => {
                              m.typ = IPCMsgType::Ok;
                              m.create_payload("Ok").unwrap();
                          }
                          IPCMsgType::Status => {
                              m.typ = IPCMsgType::Status;
                              m.create_payload("Running").unwrap();
                          }
                          IPCMsgType::Statistic => {
                              m.typ = IPCMsgType::Statistic;
                              let mut ret: String = String::with_capacity(1024);
                              for s in &stats {
                                  ret.push_str(&s.to_string());
                              }
                              m.create_payload(&ret).unwrap();
                          }
                          IPCMsgType::Quit => {
                              m.typ = IPCMsgType::Quit;
                              m.create_payload("Server shutting down").unwrap();
                              ipc.send_msg(m).unwrap();
                              ipc.shutdown();
                              process::exit(0);
                          }
                          _ => {
                              m.typ = IPCMsgType::Any;
                              m.create_payload("Placeholder").unwrap();
                          },
                      }
                      ipc.send_msg(m).unwrap();
                  });
}

fn main() {
    env_logger::init().unwrap();
    uidcheck();

    let matches = App::new("flatlined - a heartbeat daemon")
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

    let mut stats: Vec<Statistic> = Vec::with_capacity(opts.server.len());
    for s in &opts.server {
        stats.push(Statistic::new(s));
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

    ipc_handler(stats.clone());

    //determine mode:
    //client - no servers were defined in the config
    //server - at least one server was defined in the config

    //thread signal via channels to stop when ipc gets an exit
    if opts.server.len() == 0 {
        let socket = BeatListenSocket::new(&opts);
        thread::spawn(move || loop {
                          match socket.listen() {
                              Ok(_) => println!("Message received and Ok!"),
                              Err(_) => println!("Error!"),
                          }
                      });
    } else {
        let send = BeatSendSocket::new(&opts);
        let recv = BeatListenSocket::new(&opts);

        thread::spawn(move || loop {
                          match recv.listen() {
                              Ok(_) => println!("Message received and Ok!"),
                              Err(_) => println!("Error!"),
                          }
                      });

        loop {
            send.send_all();
        }
    }

}
