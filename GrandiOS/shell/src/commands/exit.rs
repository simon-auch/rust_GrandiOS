use utils::parser::Argument;
use core::result::Result;
use alloc::string::String;
use alloc::vec_deque::VecDeque;

pub fn exec(mut args: VecDeque<Argument>) -> Result<VecDeque<Argument>, String> {
    exit!();
    Ok(VecDeque::new())
}
