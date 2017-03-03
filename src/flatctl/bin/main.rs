#[macro_use]
extern crate log;
extern crate env_logger;
extern crate ipc;

use std::env;
use std::process;

use ipc::*;

static FLATSOCK: &'static str = "ipc:///var/run/flatlined.sock";

fn help() -> () {
    println!("status|quit|none");
    process::exit(1);
}

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
    match m.typ {
        IPCMsgType::Ok => println!("Ok received!"),
        _ => println!("Shits received!"),
    };
    println!("{}", std::str::from_utf8(&m.msg).unwrap());

    process::exit(0);
}

fn main() {
    env_logger::init().unwrap();
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        help();
    }
    let mut ipc = IPC::new_connect(FLATSOCK);

    if args[1] == "status" {
        let msg = IPCMsg {
            typ: IPCMsgType::Status,
            msg: [0u8; 1024],
        };
        communicate(&mut ipc, msg);
    }
}
