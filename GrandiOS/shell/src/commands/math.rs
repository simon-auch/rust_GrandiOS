use utils::parser::Argument;
use core::result::Result;
use alloc::string::{String,ToString};
use alloc::vec::Vec;

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

pub fn operate_diad<F>(mut args: Vec<Argument>, f: F) -> Result<Vec<Argument>, String> where F: Fn(isize, isize) -> isize {
    match args.len() {
        1 | 2 => return Ok(args),
        3 => if !args[0].is_something() { return Ok(args) },
        _ => {}
    }
    args.remove(1);
    ::eval_args(&mut args, 2);
    if !args[0].is_something() { args[0] = args.remove(2); }
    if args[0].is_method() { args[0] = Argument::Application(vec![args[0].clone()]); }
    ::eval_args(&mut args, 1);
    if !args[0].is_int() || !args[1].is_int() { return Err("Ints expected".to_string()); }
    let r = Argument::Int(f(args[0].get_int().unwrap(),args[1].get_int().unwrap()));
    args.remove(0);
    args[0] = r;
    Ok(args)
}
