use utils::parser::Argument;
use utils::evaluate::*;
use core::str;
use core::fmt::Write;
use core::result::Result;
use alloc::string::{ToString,String};
use alloc::vec_deque::VecDeque;
use alloc::vec::Vec;

pub fn exec(mut args: VecDeque<Argument>) -> Result<VecDeque<Argument>, String> {
    if help_call(&args) {
        println!("If Barry got a question it will wait for 5 minutes before terminating");
        return Ok(VecDeque::new());
    }
    if args.len() < 2 { return Ok(args); }
    args.pop_front();
    if !args[0].is_str() { return Err("String expected".to_string()); }
    let w = args.pop_front().unwrap().get_str().unwrap();
    if *w.as_bytes().last().unwrap() == 63 {
        let timeout = 100;
        let mut buffer = vec![];
        for i in 0..(1000*60*5/timeout) {
            let r = read!(timeout);
            if r.is_none() { continue; }
            let c = r.unwrap();
            if (c as char) == '\r' || (c as char) == '\n' {
                match str::from_utf8(&buffer[..]).unwrap() {
                    "kommt drauf an" => {
                        println!("\nWorauf?");
                        return Ok(args);
                    },
                    "geld" | "spiele" => {
                        return Ok(args);
                    },
                    "spaß" | "komfort" => {
                        sleep!(1000*10);
                        return Ok(args);
                    },
                    "abstraktion" | "ressourcenverwaltung" => {
                        return Ok(args);
                    },
                    _ => { println!(""); }
                }
                buffer = vec![];
            } else {
                print!("{}", c as char);
                buffer.push(c);
            }
        }
    }
    Ok(args)
}
