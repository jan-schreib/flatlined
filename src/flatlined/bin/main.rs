#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate log;

extern crate trust_dns_resolver;
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
use server::Server;
use std::process;
use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

use std::fs;
use std::os::unix::fs::PermissionsExt;

static DEFAULT_CONF: &'static str = "/etc/flat.conf";
static PIDFILE: &'static str = "/var/run/flatlined.pid";
static FLATUSER: &'static str = "_flatlined";
static FLATGROUP: &'static str = "_flatlined";
static FLATSOCKPATH: &'static str = "/var/run/flatlined.sock";

fn uidcheck() -> () {
    if unistd::geteuid() != 0 {
        error!("Starting this application requires root privileges");
        process::exit(1);
    } else {
        return;
    }
}

fn ipc_handler(statistic: &[Statistic], rx: Receiver<Statistic>, flatsock: &str) -> () {
    let prefix = "ipc://".to_string();
    let mut ipc = IPC::new_bind(&(prefix + flatsock));
    let mut stats = statistic.to_vec();
    let meta = fs::metadata(flatsock).unwrap();
    let mut permissions = meta.permissions();
    permissions.set_mode(0o666);
    fs::set_permissions(flatsock, permissions).unwrap();

    thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));

        match rx.try_iter().last() {
            Some(v) => {
                if stats.is_empty() {
                    stats.push(v.clone());
                }
                match stats.iter().position(|ref mut x| x.server == v.server) {
                    Some(x) => {
                        stats[x].send_beats = v.send_beats;
                        stats[x].recv_beats = v.recv_beats;
                    }
                    None => {
                        stats.push(v.clone());
                    }
                }
            }
            None => (),
        };

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
                if stats.is_empty() {
                    m.create_payload("Building statistics...").unwrap();
                } else {
                    for s in &stats {
                        ret.push_str(&s.to_string());
                    }
                    m.create_payload(&ret).unwrap();
                }
            }
            IPCMsgType::Quit => {
                m.typ = IPCMsgType::Quit;
                m.create_payload("Server shutting down").unwrap();
                ipc.send_msg(m).unwrap();
                ipc.shutdown();
                //remove sock file
                process::exit(0);
            }
            _ => {
                m.typ = IPCMsgType::Any;
                m.create_payload("Placeholder").unwrap();
            }
        }
        ipc.send_msg(m).unwrap();
    });
}

fn main() {
    env_logger::init().unwrap();
    uidcheck();
    let sr_thread: JoinHandle<_>;

    let matches = App::new("flatlined - a heartbeat daemon")
        .version("0.1")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("debug")
                .short("d")
                .long("debug")
                .help("Debug mode, don't detach or become a daemon.")
                .takes_value(false),
        )
        .get_matches();

    let opts: FlatConf;
    match matches.value_of("config") {
        Some(x) => {
            match FlatConf::parse_file(x.to_owned()) {
                Ok(conf) => opts = conf,
                Err(err) => {
                    error!("{}", err.to_string());
                    process::exit(1);
                }
            }
        }
        None => {
            match FlatConf::parse_file(DEFAULT_CONF.to_owned()) {
                Ok(conf) => opts = conf,
                Err(err) => {
                    error!("{}", err.to_string());
                    process::exit(1)
                }
            }
        }
    }

    let servers: Vec<Server>;
    let nopts = opts.clone();

    match nopts.server {
        Some(x) => servers = x.clone(),
        None => servers = Vec::new(),
    }

    let mut stats: Vec<Statistic> = Vec::new();
    if !servers.is_empty() {
        for s in &servers {
            stats.push(Statistic::new(s));
        }
    }

    //become a daemon
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

    let (tx, rx): (Sender<Statistic>, Receiver<Statistic>) = mpsc::channel();
    ipc_handler(
        &stats,
        rx,
        &opts.clone().socket.unwrap_or(FLATSOCKPATH.to_string()),
    );

    //determine mode:
    //server - no servers were defined in the config
    //server receives beats from clients. the clients know the key of the server.
    //client - at least one server was defined in the config
    //client sends beats to each server defined
    //thread signal via channels to stop when ipc gets an exit

    if servers.is_empty() {
        let socket = BeatListenSocket::new(&opts);
        sr_thread = thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_millis(1000));
            match socket.listen() {
                Ok((beat, ip)) => {
                    match beat.verify_beat(&opts.key) {
                        Ok(_) => {
                            match stats.iter().position(
                                |ref mut x| x.server.address == ip.to_string(),
                            ) {
                                Some(x) => {
                                    stats[x].incr_recv();
                                    tx.send(stats[x].clone()).unwrap();
                                }
                                None => {
                                    stats.push(Statistic {
                                        recv_beats: 1,
                                        send_beats: 0,
                                        server: Server {
                                            address: ip.to_string(),
                                            port: 0,
                                            key: "".to_string(),
                                        },
                                    });
                                    tx.send(stats.last().unwrap().clone()).unwrap()
                                }
                            };
                        }
                        Err(_) => println!("Could not verifiy beat"),
                    }
                }
                Err(_) => println!("Error!"),
            }
        });
    } else {
        let send = BeatSendSocket::new(&opts);

        sr_thread = thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_millis(1000));
            for (i, s) in send.conf.server.clone().iter().enumerate() {
                match send.send(s[i].key.clone(), s[i].address.clone(), s[i].port) {
                    Ok(_) => {
                        stats[i].incr_send();
                        tx.send(stats[i].clone()).unwrap();
                    }
                    Err(_) => error!("Send error!"),
                }
            }
        });

    }
    sr_thread.join().unwrap()
}
