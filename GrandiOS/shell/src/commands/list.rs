use utils::parser::Argument;
use core::result::Result;
use alloc::string::{String,ToString};
use alloc::vec::Vec;
use commands::higher;

pub fn populate(commands: &mut Vec<(Argument, fn(Vec<Argument>) -> Result<Vec<Argument>,String>)>) {
    commands.push(command!(Method, "filter", filter));
    commands.push(command!(Method, "head", head));
    commands.push(command!(Method, "tail", tail));
    commands.push(command!(Operator, "++", plusplus));
}

pub fn filter(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    if args.len() < 3 { return Ok(args); }
    args.remove(0);
    ::unpack_args(&mut args, 2);
    if !args[1].is_list() { return Err("Arg2: List expected".to_string()); }
    let mut res = vec![];
    for e in args[1].get_list() {
        let mut cmd = higher::get_cmd(&mut args, e.clone());
        match ::apply(&mut Argument::Application(cmd.clone())) {
            Some(r) => {if r.is_bool() && r.get_bool().unwrap() { res.push(e.clone()); }},
            None => return Err(format!("Executing {}  failed", Argument::Application(cmd).to_string()))
        }
    }
    args.remove(0);
    args[0] = Argument::List(res);
    Ok(args)
}

pub fn plusplus(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    if args.len() < 3 { return Ok(args); }
    args.remove(1);
    ::unpack_args(&mut args, 2);
    if !args[1].is_list() || !args[0].is_list() { return Err("Lists expected".to_string()); }
    let mut res = args[0].get_list();
    res.append(&mut args[1].get_list());
    args.remove(0);
    args[0] = Argument::List(res);
    Ok(args)
}

pub fn head(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    if args.len() < 2 { return Ok(args); }
    args.remove(0);
    if !args[0].is_list() { return Err("List expected".to_string()); }
    args[0] = args[0].get_list()[0].clone();
    Ok(args)
}

pub fn tail(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    if args.len() < 2 { return Ok(args); }
    args.remove(0);
    if !args[0].is_list() { return Err("List expected".to_string()); }
    args[0] = Argument::List(args[0].get_list()[1..].to_vec());
    Ok(args)
}
