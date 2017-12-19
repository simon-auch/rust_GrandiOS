use utils::parser::Argument;
use utils::evaluate::*;
use core::result::Result;
use core::fmt::Write;
use alloc::string::String;
use alloc::vec_deque::VecDeque;

pub fn exec(mut args: VecDeque<Argument>) -> Result<VecDeque<Argument>, String> {
    if help_call(&args) {
        println!("please don't");
        return Ok(VecDeque::new());
    }
    exit!();
    Ok(VecDeque::new())
}
