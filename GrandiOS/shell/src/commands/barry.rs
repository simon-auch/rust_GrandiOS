use utils::parser::Argument;
use utils::evaluate::*;
use core::fmt::Write;
use core::result::Result;
use alloc::string::{ToString,String};
use alloc::vec_deque::VecDeque;

pub fn exec(mut args: VecDeque<Argument>) -> Result<VecDeque<Argument>, String> {
    if help_call(&args) {
        println!("If Barry got a question it will wait for 5 minutes before terminating");
        return Ok(VecDeque::new());
    }
    if args.len() < 2 { return Ok(args); }
    args.pop_front();
    if !args[0].is_str() { return Err("String expected".to_string()); }
    let w = args.pop_front().unwrap().get_str().unwrap();
    if *w.as_bytes().last().unwrap() == 63 { sleep!(1000*60*5); }
    Ok(args)
}
