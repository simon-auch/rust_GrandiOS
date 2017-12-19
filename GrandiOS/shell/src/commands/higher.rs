use utils::parser::Argument;
use utils::evaluate::*;
use core::result::Result;
use core::fmt::Write;
use alloc::string::{String,ToString};
use alloc::vec::Vec;
use alloc::vec_deque::VecDeque;

pub fn populate(commands: &mut Vec<(Argument, fn(VecDeque<Argument>) -> Result<VecDeque<Argument>,String>)>) {
    commands.push(command!(Method, "map", map));
    commands.push(command!(Method, "foldl", foldl));
    commands.push(command!(Method, "fix", fix));
    commands.push(command!(Operator, ".", dot));
    commands.push(command!(Operator, "\\", lambda));
    commands.push(command!(Operator, "->", lambda));
}

pub fn map(mut args: VecDeque<Argument>) -> Result<VecDeque<Argument>, String> {
    if help_call(&args) {
        println!("applies a function to each element of a list");
        return Ok(VecDeque::new());
    }
    if args.len() < 3 { return Ok(args); }
    args.pop_front();
    eval_args(&mut args, 2);
    if !args[1].is_list() { return Err("Arg2: List expected".to_string()); }
    let mut res = vec![];
    for e in args[1].get_list() {
        let mut cmd = get_cmd(&mut args, e, false);
        match apply(&mut Argument::Application(cmd.clone())) {
            Some(r) => res.push(r),
            None => return Err(format!("Executing {}  failed", Argument::Application(cmd).to_string()))
        }
    }
    args.pop_front();
    args[0] = Argument::List(res);
    Ok(args)
}

pub fn fix(mut args: VecDeque<Argument>) -> Result<VecDeque<Argument>, String> {
    if help_call(&args) {
        println!("calls a function with itself as first argument");
        return Ok(VecDeque::new());
    }
    if args.len() < 3 { return Ok(args); }
    let f = args.pop_front().unwrap();
    eval_args(&mut args, 1);
    if !args[0].is_application() { args[0] = Argument::Application(VecDeque::from(vec![args[0].clone()])); }
    let mut arg = args[0].get_application();
    arg = get_cmd(&mut arg, Argument::Application(VecDeque::from(vec![f, args[0].clone()])), true);
    arg.append(&mut args.split_off(1));
    let cmd = Argument::Application(arg.clone());
    match apply(&mut cmd.clone()) {
        Some(r) => Ok(VecDeque::from(vec![r])),
        None => Err(format!("Executing {}  failed", cmd.to_string()))
    }
}

pub fn foldl(mut args: VecDeque<Argument>) -> Result<VecDeque<Argument>, String> {
    if help_call(&args) {
        println!("folds a given list to a scalar");
        return Ok(VecDeque::new());
    }
    if args.len() < 4 { return Ok(args); }
    args.pop_front();
    eval_args(&mut args, 3);
    if !args[2].is_list() { return Err("Arg3: List expected".to_string()); }
    let mut akk = args.swap_remove_front(1).unwrap();
    for e in args[1].get_list() {
        let mut cmd = if args[0].is_application() && args[0].get_application().len() == 2 && args[0].get_application()[1].is_operator() && !args[0].get_application()[0].is_something() {
            get_cmd(&mut VecDeque::from(vec![Argument::Application(VecDeque::from(vec![akk.clone(), args[0].get_application()[1].clone()]))]), e, false)
        } else {
            VecDeque::from(vec![args[0].clone(), akk.clone(), e])
        };
        match apply(&mut Argument::Application(cmd.clone())) {
            Some(r) => akk = r,
            None => return Err(format!("Executing {} failed", Argument::Application(cmd).to_string()))
        }
    }
    args.pop_front();
    args[0] = akk;
    Ok(args)
}

pub fn get_cmd(args: &mut VecDeque<Argument>, e: Argument, second: bool) -> VecDeque<Argument> {
    if is_var(&args[0]) {
        let mut t = args.clone();
        let v = t.pop_front().unwrap().get_method_name().unwrap();
        let mut res = VecDeque::from(vec![get_var(v)]);
        res.append(&mut t);
        return get_cmd(&mut res, e, second);
    }
    if args[0].is_application() {
        let mut arg = args[0].get_application();
        if arg.len() > 2 && !arg[0].is_something() {
            arg[0] = e.clone();
            arg
        } else if arg.len() >= 2 && arg[1].is_operator() {
            if second {
                arg.insert(1, e.clone());
            } else {
                arg.push_back(e.clone());
            }
            arg
        } else if arg.len() == 1 {
            VecDeque::from(vec![e.clone(), args[0].clone()])
        } else {
            VecDeque::from(vec![args[0].clone(), e.clone()])
        }
    } else if !args[0].is_something() && second {
        args[0] = e.clone();
        args.clone()
    } else {
        VecDeque::from(vec![args[0].clone(), e.clone()])
    }
}

pub fn dot(mut args: VecDeque<Argument>) -> Result<VecDeque<Argument>, String> {
    if args.len() < 4 { return Ok(args); }
    args.swap_remove_front(1);
    unpack_args(&mut args, 3);
    let f1 = args.pop_front().unwrap();
    let f2 = args.pop_front().unwrap();
    let mut cmd = VecDeque::from(vec![f1, Argument::Application(VecDeque::from(vec![f2, args[0].clone()]))]);
    unpack_args(&mut cmd, 0);
    match apply(&mut Argument::Application(cmd.clone())) {
        Some(r) => args[0] = r,
        None => return Err(format!("Executing {} failed", Argument::Application(cmd).to_string()))
    }
    Ok(args)
}

pub fn lambda(mut args: VecDeque<Argument>) -> Result<VecDeque<Argument>, String> {
    if args[1].get_operator().unwrap() != "\\".to_string() { return Ok(args); }
    if !args[0].is_something() && args.len() < 4 { return Ok(args); }
    if args.len() < 3 { return Ok(args); }
    if !args[0].is_something() { args[0] = args.remove(3).unwrap(); }
    let mut arg = args[2].get_application();
    let mut lv = arg[0].get_application();
    eval_args(&mut args, 1);
    set_var_local(lv[0].get_method_name().unwrap(), &args[0]);
    let mut cmd = if lv.len() > 1 {
        Argument::Application(VecDeque::from(vec![Argument::Nothing, Argument::Operator("\\".to_string()), Argument::Application(VecDeque::from(vec![Argument::Application(lv.split_off(1)), Argument::Operator("->".to_string()), arg[2].clone()]))]))
    } else { arg[2].clone() };
    if args.len() > 3 {
        let mut cmdargs = VecDeque::from(vec![cmd]);
        cmdargs.append(&mut args.split_off(3));
        cmd = Argument::Application(cmdargs);
    }
    match apply(&mut cmd) {
        Some(a) => Ok(VecDeque::from(vec![a])),
        None => Err("Application failed".to_string())
    }
}
