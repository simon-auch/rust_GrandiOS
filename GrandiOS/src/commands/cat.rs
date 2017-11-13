use driver::serial::*;
use alloc::vec::Vec;

pub fn exec(args: Vec<&str>) {
    let mut c = read!();
    while c != 4 { //4 = ^d = end of transmission
        if args.len() == 0 {
            print!("{}", c as char);
        } else {
            print!("{}", c);
        }
        c = read!();
    }
}
