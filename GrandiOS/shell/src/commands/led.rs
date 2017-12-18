use utils::parser::Argument;
use utils::evaluate::*;
use core::result::Result;
use alloc::string::{String,ToString};
use alloc::vec::Vec;
use alloc::vec_deque::VecDeque;

pub fn populate(commands: &mut Vec<(Argument, fn(VecDeque<Argument>) -> Result<VecDeque<Argument>,String>)>) {
    commands.push(command!(Method, "get_led", get_led));
    commands.push(command!(Method, "set_led", set_led));
}

pub fn get_led(mut args: VecDeque<Argument>) -> Result<VecDeque<Argument>, String> {
    if args.len() < 2 { return Ok(args); }
    args.pop_front();
    eval_args(&mut args, 1);
    if !args[0].is_int() || args[0].get_int().unwrap() > 2 || args[0].get_int().unwrap() < 0 { return Err("Invalid LED".to_string()); }
    let led = args[0].get_int().unwrap() as u8;
    args[0] = Argument::Bool(get_led!(led));
    Ok(args)
}

pub fn set_led(mut args: VecDeque<Argument>) -> Result<VecDeque<Argument>, String> {
    if args.len() < 3 { return Ok(args); }
    args.pop_front();
    args[1] = Argument::Application(VecDeque::from(vec![args[1].clone()]));
    eval_args(&mut args, 2);
    if !args[0].is_int() || args[0].get_int().unwrap() > 2 || args[0].get_int().unwrap() < 0 { return Err("Invalid LED".to_string()); }
    if !args[1].is_bool() { return Err("State is expected to be bool".to_string()); }
    let led = args.pop_front().unwrap().get_int().unwrap() as u8;
    let state = args.pop_front().unwrap().get_bool().unwrap();
    set_led!(led, state);
    Ok(args)
}
