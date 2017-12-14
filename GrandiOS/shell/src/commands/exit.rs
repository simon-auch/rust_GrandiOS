use core::fmt::Write;
use utils::vt;
use utils::parser::Argument;
use core::result::Result;
use alloc::string::String;
use alloc::vec::Vec;

pub fn exec(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    exit!();
    Ok(vec![])
}
