use utils::parser::Argument;
use core::result::Result;
use alloc::string::{String,ToString};
use alloc::vec::Vec;

pub fn map(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    if args.len() < 3 { return Ok(args); }
    args.remove(0);
    ::unpack_args(&mut args, 2);
    if !args[1].is_list() { return Err("Arg2: List expected".to_string()); }
    let mut res = vec![];
    for e in args[1].get_list() {
        let mut cmd = get_cmd(&mut args, e);
        match ::apply(&mut Argument::Application(cmd.clone())) {
            Some(r) => res.push(r),
            None => return Err(format!("Executing {}  failed", Argument::Application(cmd).to_string()))
        }
    }
    args.remove(0);
    args[0] = Argument::List(res);
    Ok(args)
}

pub fn foldl(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    if args.len() < 4 { return Ok(args); }
    args.remove(0);
    ::unpack_args(&mut args, 3);
    if !args[2].is_list() { return Err("Arg3: List expected".to_string()); }
    let mut akk = args.remove(1);
    for e in args[1].get_list() {
        let mut cmd = if args[0].is_application() && args[0].get_application().len() == 2 && args[0].get_application()[1].is_operator() && !args[0].get_application()[0].is_something() {
            get_cmd(&mut vec![Argument::Application(vec![akk.clone(), args[0].get_application()[1].clone()])], e)
        } else {
            vec![args[0].clone(), akk.clone(), e]
        };
        match ::apply(&mut Argument::Application(cmd.clone())) {
            Some(r) => akk = r,
            None => return Err(format!("Executing {} failed", Argument::Application(cmd).to_string()))
        }
    }
    args.remove(0);
    args[0] = akk;
    Ok(args)
}

pub fn get_cmd(args: &mut Vec<Argument>, e: Argument) -> Vec<Argument> {
    if ::is_var(&args[0]) {
        let mut t = args.clone();
        let v = t.remove(0).get_method_name().unwrap();
        let mut res = vec![::get_var(v)];
        res.append(&mut t);
        return get_cmd(&mut res, e);
    }
    if args[0].is_application() {
        let mut arg = args[0].get_application();
        if arg.len() > 2 && !arg[0].is_something() {
            arg[0] = e.clone();
            arg
        } else if arg.len() >= 2 && arg[1].is_operator() {
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
    if args.len() < 4 { return Ok(args); }
    args.remove(1);
    ::unpack_args(&mut args, 3);
    let f1 = args.remove(0);
    let f2 = args.remove(0);
    let mut cmd = vec![f1, Argument::Application(vec![f2, args[0].clone()])];
    ::unpack_args(&mut cmd, 0);
    match ::apply(&mut Argument::Application(cmd.clone())) {
        Some(r) => args[0] = r,
        None => return Err(format!("Executing {} failed", Argument::Application(cmd).to_string()))
    }
    Ok(args)
}

pub fn lambda(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    if args[1].get_operator().unwrap() != "\\".to_string() { return Ok(args); }
    if !args[0].is_something() && args.len() < 4 { return Ok(args); }
    if args.len() < 3 { return Ok(args); }
    if !args[0].is_something() { args[0] = args.remove(3); }
    let mut arg = args[2].get_application();
    let lv = arg[0].get_application();
    ::set_var_local(lv[0].get_method_name().unwrap(), &args[0]);
    let mut cmd = if lv.len() > 1 {
        Argument::Application(vec![Argument::Nothing, Argument::Operator("\\".to_string()), Argument::Application(vec![Argument::Application(lv[1..].to_vec()), Argument::Operator("->".to_string()), arg[2].clone()])])
    } else { arg[2].clone() };
    if args.len() > 3 {
        let mut cmdargs = vec![cmd];
        cmdargs.append(&mut args[3..].to_vec());
        cmd = Argument::Application(cmdargs);
    }
    match ::apply(&mut cmd) {
        Some(a) => Ok(vec![a]),
        None => Err("Application failed".to_string())
    }
}
