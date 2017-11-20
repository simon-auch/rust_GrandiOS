use driver::serial::*;
use utils::parser::Argument;
use utils::shell;
use core::result::Result;
use alloc::string::{String,ToString};
use alloc::vec::Vec;

pub fn map(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    if args.len() < 2 { return Ok(args); } //TODO
    shell::unpack_args(&mut args, 2);
    if !args[1].is_list() { return Err("Arg2: List expected".to_string()); }
    let mut res = vec![];
    for e in args[1].get_list() {
        let mut cmd = get_cmd(&mut args, e);
        match shell::apply(&mut Argument::Application(cmd.clone())) {
            Some(r) => res.push(r),
            None => return Err(format!("Executing {}  failed", Argument::Application(cmd).to_string()))
        }
    }
    args.remove(0);
    args[0] = Argument::List(res);
    Ok(args)
}

pub fn foldl(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    if args.len() < 3 { return Ok(args); } //TODO
    shell::unpack_args(&mut args, 3);
    if !args[2].is_list() { return Err("Arg3: List expected".to_string()); }
    let mut akk = args.remove(1);
    for e in args[1].get_list() {
        let mut cmd = if args[0].is_application() && args[0].get_application().len() == 2 && args[0].get_application()[1].is_operator() && !args[0].get_application()[0].is_something() {
            vec![Argument::Application(vec![akk.clone(), args[0].get_application()[1].clone()])]
        } else {
            vec![args[0].clone(), e.clone()]
        };
        cmd = get_cmd(&mut cmd, e);
        match shell::apply(&mut Argument::Application(cmd.clone())) {
            Some(r) => akk = r,
            None => return Err(format!("Executing {} failed", Argument::Application(cmd).to_string()))
        }
    }
    args.remove(0);
    args[0] = akk;
    Ok(args)
}

fn get_cmd(args: &mut Vec<Argument>, e: Argument) -> Vec<Argument> {
    if args[0].is_application() {
        let mut arg = args[0].get_application();
        if arg.len() > 2 && !arg[0].is_something() {
            arg[0] = e.clone();
            arg
        } else if arg.len() == 2 && arg[1].is_operator() {
            arg.push(e.clone());
            arg
        } else if arg.len() == 1 {
            vec![e.clone(), args[0].clone()]
        } else {
            vec![args[0].clone(), e.clone()]
        }
    } else {
        vec![args[0].clone(), e.clone()]
    }
}

pub fn dot(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    if args.len() < 3 { return Ok(args); }
    shell::unpack_args(&mut args, 3);
    let f1 = args.remove(0);
    let f2 = args.remove(0);
    let mut cmd = vec![f1, Argument::Application(vec![f2, args[0].clone()])];
    shell::unpack_args(&mut cmd, 0);
    match shell::apply(&mut Argument::Application(cmd.clone())) {
        Some(r) => args[0] = r,
        None => return Err(format!("Executing {} failed", Argument::Application(cmd).to_string()))
    }
    Ok(args)
}
