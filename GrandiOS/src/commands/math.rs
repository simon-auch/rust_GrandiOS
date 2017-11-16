use driver::serial::*;
use utils::parser::Argument;
use core::result::Result;
use alloc::string::{String,ToString};
use alloc::vec::Vec;

pub fn plus(args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    if args.len() < 2 { return Err("Too few arguments".to_string()); }
    if !args[0].is_int() || !args[1].is_int() { return Err("Ints expected".to_string()); }
    Ok(vec![Argument::Int(args[0].get_int().unwrap()+args[1].get_int().unwrap())])
}
