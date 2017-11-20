use utils::parser::Argument;
use core::result::Result;
use alloc::string::{String,ToString};
use alloc::vec::Vec;

pub fn inc(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    operate_monad(args, |x| x+1)
}

pub fn dec(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    operate_monad(args, |x| x-1)
}

pub fn plus(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    operate_diad(args, |x,y| x+y)
}

pub fn minus(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    operate_diad(args, |x,y| x-y)
}

pub fn times(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    operate_diad(args, |x,y| x*y)
}

pub fn div(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    operate_diad(args, |x,y| x/y)
}

pub fn operate_monad<F>(mut args: Vec<Argument>, f: F) -> Result<Vec<Argument>, String> where F: Fn(usize) -> usize {
    if args.len() < 1 { return Err("Too few arguments".to_string()); }
    if !args[0].is_int() { return Err("Int expected".to_string()); }
    let r = Argument::Int(f(args[0].get_int().unwrap()));
    args[0] = r;
    Ok(args)
}

pub fn operate_diad<F>(mut args: Vec<Argument>, f: F) -> Result<Vec<Argument>, String> where F: Fn(usize, usize) -> usize {
    if args.len() < 2 { return Err("Too few arguments".to_string()); }
    if !args[0].is_int() || !args[1].is_int() { return Err("Ints expected".to_string()); }
    let r = Argument::Int(f(args[0].get_int().unwrap(),args[1].get_int().unwrap()));
    args.remove(0);
    args[0] = r;
    Ok(args)
}
