use utils::parser::Argument;
use utils::evaluate::*;
use core::result::Result;
use alloc::string::{String,ToString};
use alloc::vec::Vec;
use alloc::vec_deque::VecDeque;

static mut CACHE: Option<Vec<(VecDeque<Argument>, VecDeque<Argument>)>> = None;

pub fn exec(mut args: VecDeque<Argument>) -> Result<VecDeque<Argument>, String> {
    unsafe { if CACHE.is_none() { CACHE = Some(vec![]); } }
    args.pop_front();
    eval_args(&mut args, 0);
    unsafe { match find(&args) { Some(res) => return Ok(res), None => {} } }
    let key = args.clone();
    match apply(&mut Argument::Application(args)) {
        None => Err("apply failed".to_string()),
        Some(res) => Ok(unsafe { add(key, VecDeque::from(vec![res])) })
    }
}

unsafe fn find(key: &VecDeque<Argument>) -> Option<VecDeque<Argument>> {
    for &(ref k, ref v) in CACHE.as_ref().unwrap().iter() {
        if k != key { continue; }
        return Some(v.clone());
    }
    None
}

unsafe fn add(key: VecDeque<Argument>, res: VecDeque<Argument>) -> VecDeque<Argument> {
    CACHE.as_mut().unwrap().push((key, res.clone()));
    res
}
