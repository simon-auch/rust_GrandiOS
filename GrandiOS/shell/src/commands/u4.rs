use core::fmt::Write;
use utils::parser::Argument;
use core::result::Result;
use alloc::string::String;
use alloc::vec_deque::VecDeque;
use alloc::vec::Vec;

pub fn exec(mut args: VecDeque<Argument>) -> Result<VecDeque<Argument>, String> {
    loop {
        let c = read!();
        if c == 4 { //^d
            break;
        }
        spawn!(thread as *const (), 0x400, c as u32);
    }
    Ok(VecDeque::new())
}

pub extern fn thread(c: u8) {
    thread_main(c);
}
#[inline(never)]
fn thread_main(c: u8) {
    for j in 0..10 {
        if (c >= 65) && (c <= 90) {
            for i in 0..100000 {}
        }
        if (c >= 97) && (c <= 122) {
            sleep!(500);
        }
        print!("{}", c as char);
    }
    println!("!");
    exit!();
}
