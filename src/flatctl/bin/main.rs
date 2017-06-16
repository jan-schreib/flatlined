#[macro_use]
extern crate log;
extern crate clap;
extern crate env_logger;
extern crate ipc;

use std::{process, fs, str};
use clap::{Arg, App};
use ipc::*;

static FLATSOCK: &'static str = "ipc:///var/run/flatlined.sock";
static FLATSOCKPATH: &'static str = "/var/run/flatlined.sock";

fn communicate(ipc: &mut IPC, msg: IPCMsg) -> () {

    ipc.set_send_timeout(2000);
    ipc.set_recv_timeout(2000);

    match ipc.send_msg(msg) {
        Ok(_) => {}
        Err(err) => {
            error!("{}", err);
            process::exit(1);
        }
    }

    let m: IPCMsg;
    match ipc.receive_msg() {
        Ok(msg) => m = msg,
        Err(msg) => {
            error!("{}", msg);
            process::exit(1);
        }
    }

    let response = str::from_utf8(&m.msg).unwrap();
    match m.typ {
        IPCMsgType::Ok => println!("{}", response),
        IPCMsgType::Statistic => println!("{}", response),
        IPCMsgType::Status => println!("{}", response),
        IPCMsgType::Quit => println!("{}", response),
        _ => println!("{}", response),
    };

    process::exit(0);
}

fn main() {
    env_logger::init().unwrap();

    let matches = App::new("flatctl - the tool for controlling the heartbeat daemon")
        .version("0.1")
        .arg(Arg::with_name("command")
                 .short("c")
                 .long("command")
                 .value_name("COMMAND")
                 .help("Valid commands: ok (default), status, statistic, quit, any")
                 .takes_value(true))
        .get_matches();

    let mut ipc: IPC;
    match fs::metadata(FLATSOCKPATH) {
        Ok(_) => ipc = IPC::new_connect(FLATSOCK),
        Err(_) => {
            error!("IPC Socket not found.");
            process::exit(1);
        }
    }

    let com: &str;
    let msg_type: IPCMsgType;
    match matches.value_of("command") {
        Some(x) => {
            com = x;
            match x.as_ref() {
                "ok" => msg_type = IPCMsgType::Ok,
                "status" => msg_type = IPCMsgType::Status,
                "statistic" => msg_type = IPCMsgType::Statistic,
                "quit" => msg_type = IPCMsgType::Quit,
                _ => msg_type = IPCMsgType::Any,
            }
        }
        None => {
            msg_type = IPCMsgType::Ok;
            com = "ok";
        },
    }

    let mut msg = IPCMsg {
        typ: msg_type,
        msg: [0; 1024],
    };

    msg.create_payload(com).unwrap();
    communicate(&mut ipc, msg);
}
