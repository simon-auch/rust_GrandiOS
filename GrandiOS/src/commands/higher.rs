use utils::parser::Argument;
use utils::shell;
use core::result::Result;
use alloc::string::{String,ToString};
use alloc::vec::Vec;

pub fn map(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    if args.len() < 2 { return Err("Too few arguments".to_string()); }
    if !args[0].is_method() { return Err("Arg1: Method expected".to_string()); }
    if !args[1].is_list() { return Err("Arg2: List expected".to_string()); }
    let mut res = vec![];
    for e in args[1].get_list() {
        match shell::apply(&mut Argument::Application(vec![args[0].clone(), e.clone()])) {
            Some(r) => res.push(r),
            None => return Err(format!("Executing {} {} failed", args[0].get_method_name().unwrap(), e.to_string()))
        }
    }
    args.remove(0);
    args[0] = Argument::List(res);
    Ok(args)
}
