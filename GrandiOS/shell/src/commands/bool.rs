use utils::parser::Argument;
use utils::evaluate::*;
use core::result::Result;
use alloc::string::{String,ToString};
use alloc::vec::Vec;
use alloc::boxed::Box;
use core::fmt::Write;

pub fn populate(commands: &mut Vec<(Argument, fn(Vec<Argument>) -> Result<Vec<Argument>,String>)>) {
    commands.push(command!(Method, "not", not));
    commands.push(command!(Method, "if", bif));
    commands.push(command!(Operator, "==", eq));
    commands.push(command!(Operator, "/=", neq));
    commands.push(command!(Operator, "&&", band));
    commands.push(command!(Operator, "||", bor));
    commands.push(command!(Operator, "<", lesser));
    commands.push(command!(Operator, ">", greater));
    commands.push(command!(Operator, "<=", leq));
    commands.push(command!(Operator, ">=", geq));
}

pub fn eq(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    operate_diad(args, |x,y| x==y, None::<fn(_,_)->_>)
}

pub fn neq(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    operate_diad(args, |x,y| x!=y, None::<fn(_,_)->_>)
}

pub fn lesser(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    operate_diad(args, |x,y| x<y, None::<fn(_,_)->_>)
}

pub fn greater(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    operate_diad(args, |x,y| x>y, None::<fn(_,_)->_>)
}

pub fn leq(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    operate_diad(args, |x,y| x<=y, None::<fn(_,_)->_>)
}

pub fn geq(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    operate_diad(args, |x,y| x>=y, None::<fn(_,_)->_>)
}

pub fn band(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    operate_diad(args, |x,y| x.get_bool().unwrap()&&y.get_bool().unwrap(), Some::<fn(Argument,Argument)->bool>(|x,y| x.is_bool()&&y.is_bool()))
}

pub fn bor(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    operate_diad(args, |x,y| x.get_bool().unwrap()||y.get_bool().unwrap(), Some::<fn(Argument,Argument)->bool>(|x,y| x.is_bool()&&y.is_bool()))
}

pub fn bif(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    if args.len() < 6 { return Ok(args); }
    if !args[2].is_method() || args[2].get_method_name().unwrap() != "then" ||
       !args[4].is_method() || args[4].get_method_name().unwrap() != "else" {
           return Err("Invalid if syntax".to_string());
    }
    //remove the useless keywords
    args.remove(4);
    args.remove(2);
    args.remove(0);
    eval_args(&mut args, 1);
    if !args[0].is_application() { args[0] = Argument::Application(vec![args[0].clone()]); }
    match apply(&mut args[0]) {
        Some(s) => {
            if !s.is_bool() { return Err("Predicate did not return boolean".to_string()); }
            if s.get_bool().unwrap() { args[2] = args[1].clone() };
        },
        None => return Err("Evaluation of predicate failed".to_string())
    }
    //since we only return 1 argument, we need to shrink our array
    args.remove(0);
    args.remove(0);
    eval_args(&mut args, 1);
    Ok(args)
}

pub fn not(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    if args.len() < 2 { return Ok(args); }
    args.remove(0);
    eval_args(&mut args, 1);
    if args[0].is_method() { args[0] = Argument::Application(vec![args[0].clone()]); }
    eval_args(&mut args, 1);
    if !args[0].is_bool() { return Err("Booleanexpected".to_string()); }
    args[0] = Argument::Bool(!args[0].get_bool().unwrap());
    Ok(args)
}

pub fn operate_diad<F>(mut args: Vec<Argument>, f: F, p: Option<F>) -> Result<Vec<Argument>, String> where F: Fn(Argument, Argument) -> bool {
    match args.len() {
        1 | 2 => return Ok(args),
        3 => if !args[0].is_something() { return Ok(args) },
        _ => {}
    }
    args.remove(1);
    eval_args(&mut args, 2);
    if !args[0].is_something() { args[0] = args.remove(2); }
    if args[0].is_method() { args[0] = Argument::Application(vec![args[0].clone()]); }
    eval_args(&mut args, 1);
    if !p.map(|f| f(args[0].clone(), args[1].clone())).unwrap_or(true) { return Err("Argument condition not met".to_string()); }
    let r = Argument::Bool(f(args[0].clone(),args[1].clone()));
    args.remove(0);
    args[0] = r;
    Ok(args)
}
