use utils::parser::Argument;
use utils::evaluate::*;
use core::result::Result;
use alloc::string::{String,ToString};
use alloc::vec::Vec;

static mut CACHE: Option<Vec<(Vec<Argument>, Vec<Argument>)>> = None;

pub fn exec(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    unsafe { if CACHE.is_none() { CACHE = Some(vec![]); } }
    args.remove(0);
    eval_args(&mut args, 0);
    unsafe { match find(&args) { Some(res) => return Ok(res), None => {} } }
    let key = args.clone();
    match apply(&mut Argument::Application(args)) {
        None => Err("apply failed".to_string()),
        Some(res) => Ok(unsafe { add(key, vec![res]) })
    }
}

unsafe fn find(key: &Vec<Argument>) -> Option<Vec<Argument>> {
    for &(ref k, ref v) in CACHE.as_ref().unwrap().iter() {
        if k != key { continue; }
        return Some(v.clone());
    }
    None
}

unsafe fn add(key: Vec<Argument>, res: Vec<Argument>) -> Vec<Argument> {
    CACHE.as_mut().unwrap().push((key, res.clone()));
    res
}
