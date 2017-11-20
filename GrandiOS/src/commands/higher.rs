use driver::serial::*;
use utils::parser::Argument;
use utils::shell;
use core::result::Result;
use alloc::string::{String,ToString};
use alloc::vec::Vec;

pub fn map(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    if args.len() < 2 { return Err("Too few arguments".to_string()); }
    if !args[1].is_list() { return Err("Arg2: List expected".to_string()); }
    let mut res = vec![];
    for e in args[1].get_list() {
        let mut cmd = if args[0].is_application() {
            let mut arg = args[0].get_application();
            if arg.len() > 1 && arg[1].is_operator() {
                let mut t = arg.clone();
                t[0] = e.clone();
                Argument::Application(t)
            } else {
                arg.push(e.clone());
                Argument::Application(arg)
            }
        } else {
            Argument::Application(vec![args[0].clone(), e.clone()])
        };
        match shell::apply(&mut cmd) {
            Some(r) => res.push(r),
            None => return Err(format!("Executing {} {} failed", args[0].get_method_name().unwrap(), e.to_string()))
        }
    }
    args.remove(0);
    args[0] = Argument::List(res);
    Ok(args)
}
