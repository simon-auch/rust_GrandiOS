use core::fmt::Write;
use utils::parser::Argument;
use utils::vt;
use core::result::Result;
use alloc::string::{String,ToString};
use alloc::vec::Vec;

pub fn exec(mut args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    args.remove(0);
    ::eval_args(&mut args, 0);
    if args.len() == 0 { return Err("Arguments expected".to_string()); }
    print!("{}", "\n".repeat(8));
    print!("{}< ", &vt::CursorControl::Up{count: 7});
    for arg in args {
        print!("{} ", arg.to_string());
    }
    print!(">{}{}", &vt::CursorControl::Up{count: 1}, &vt::CursorControl::Left{count: 2});
    let (mut x, _) = vt::get_position();
    while x > 1 {
        print!("_{0}{1}-{0}{0}{2}", &vt::CursorControl::Left{count: 1}, &vt::CursorControl::Down{count: 2}, &vt::CursorControl::Up{count: 2});
        x -= 1;
    }
    print!("{}", &vt::CursorControl::Down{count: 3});
    print!("  \\  ^__^\r{}", &vt::CursorControl::Down{count: 1});
    print!("   \\ (oo)\\_______\r{}", &vt::CursorControl::Down{count: 1});
    print!("     (__)\\       )\\/\\\r{}", &vt::CursorControl::Down{count: 1});
    print!("         ||----w |\r{}", &vt::CursorControl::Down{count: 1});
    print!("         ||     ||\r\n");
    Ok(vec![])
}
