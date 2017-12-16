use utils::parser::*;
use utils::vt;
use commands::*;
use alloc::vec::Vec;
use alloc::vec_deque::VecDeque;
use alloc::string::{String, ToString};
use alloc::linked_list::LinkedList;
use core::result::Result;
use core::fmt::Write;

pub static mut COMMANDS: Option<Vec<(Argument, fn(VecDeque<Argument>) -> Result<VecDeque<Argument>,String>)>> = None;
pub static mut VARS: Option<Vec<(String, Argument)>> = None;
pub static mut LOCALVARS: Option<Vec<(String, Argument)>> = None;

#[macro_export]
macro_rules! command {
	( $t:tt, $o:expr, $c:tt, $m:tt ) => {
        (Argument::$t($o.to_string()), $m::$c as fn(VecDeque<Argument>) -> Result<VecDeque<Argument>, String>)
	};
	( $t:tt, $o:expr, $c:tt ) => {
        (Argument::$t($o.to_string()), $c as fn(VecDeque<Argument>) -> Result<VecDeque<Argument>, String>)
	};
}

pub fn get_size(mut args: VecDeque<Argument>) -> Result<VecDeque<Argument>, String> {
    let (w, h) = vt::get_size();
    args[0] = Argument::List(vec![Argument::Int(w as isize), Argument::Int(h as isize)]);
    Ok(args)
}
pub fn get_position(mut args: VecDeque<Argument>) -> Result<VecDeque<Argument>, String> {
    let (w, h) = vt::get_position();
    args[0] = Argument::List(vec![Argument::Int(w as isize), Argument::Int(h as isize)]);
    Ok(args)
}

pub fn populate_commands() {
    unsafe {
        COMMANDS = Some(vec![
            command!(Method, "pos", get_position),
            command!(Method, "size", get_size),
            command!(Method, "cache", exec, cache),
            command!(Method, "logo", exec, logo),
            command!(Method, "colors", exec, colors),
            command!(Method, "edit", exec, edit),
            command!(Method, "cowsay", exec, cowsay),
            command!(Method, "cat", exec, cat),
            command!(Method, "spot", exec, spot),
            command!(Method, "exit", exec, exit),
        ]);
        higher::populate(COMMANDS.as_mut().unwrap());
        bool::populate(COMMANDS.as_mut().unwrap());
        math::populate(COMMANDS.as_mut().unwrap());
        list::populate(COMMANDS.as_mut().unwrap());
        VARS = Some(vec![("it".to_string(), Argument::Nothing)]);
        load_prelude();
    }
}

unsafe fn load_prelude() {
    let d = include_str!("prelude.txt").split('\n');
    for l in d {
        let mut raw_input = LinkedList::new();
        for c in l.chars() {
            raw_input.push_back(c as u8);
        }
        if raw_input.is_empty() { continue; }
        raw_input.push_back(32);
        match parse(&mut raw_input, 0) {
            Err((s,p)) => { println!("{}^\n{}", "-".repeat(p+1), s); continue; },
            Ok(mut v) => { 
                match apply(&mut v.0[0]) {
                    Some(arg) => { set_var(v.1, &arg); }
                    None => {}
                }
            }
        }
    }
}

pub fn set_var(name: String, arg: &Argument) {
    unsafe { set_var_on(name, arg, &mut VARS); }
}
pub fn set_var_local(name: String, arg: &Argument) {
    unsafe {
        if LOCALVARS.is_none() { LOCALVARS = Some(vec![("".to_string(), Argument::Nothing)]); }
        set_var_on(name, arg, &mut LOCALVARS);
    }
}
pub fn set_var_on(name: String, arg: &Argument, vars: &mut Option<Vec<(String, Argument)>>) {
    let mut p: isize = -1;
    for (i, &(ref n, ref a)) in vars.as_ref().unwrap().iter().enumerate() {
        if name == *n {
            p = i as isize;
            break;
        }
    }
    if p >= 0 {
        let x = vars.as_mut().unwrap();
        x[p as usize] = (name, arg.clone());
    } else {
        vars.as_mut().unwrap().push((name, arg.clone()));
    }
}

pub fn get_var(name: String) -> Argument {
    let r = unsafe { get_var_local(name.clone(), &VARS) };
    if r == Argument::Nothing {
        unsafe { get_var_local(name, &LOCALVARS) }
    } else { r }
}
fn get_var_local(name: String, vars: &Option<Vec<(String, Argument)>>) -> Argument {
    if vars.is_none() { return Argument::Nothing; }
    for &(ref n, ref a) in vars.as_ref().unwrap().iter() {
        if name == *n { return a.clone(); }
    }
    Argument::Nothing
}

pub fn is_var(name: &Argument) -> bool {
    unsafe { is_var_local(name, &VARS) || is_var_local(name, &LOCALVARS) }
}
fn is_var_local(name: &Argument, vars: &Option<Vec<(String, Argument)>>) -> bool {
    if !name.is_method() || vars.is_none() { return false; }
    unsafe {
        for &(ref n, ref a) in vars.as_ref().unwrap().iter() {
            if name.get_method_name().unwrap() == *n { return true; }
        }
    }
    false
}

pub fn get_function(command: Argument) -> Option<fn(VecDeque<Argument>) -> Result<VecDeque<Argument>,String>> {
    unsafe {
        for &(ref c, m) in COMMANDS.as_ref().unwrap().iter() {
            if command == *c {
                return Some(m);
            }
        }
    }
    None
}

pub fn unpack_args(args: &mut VecDeque<Argument>, len: usize) {
    for i in 0..(if len > 0 && len <= args.len() { len } else { args.len() }) {
        if is_var(&args[i]) { args[i] = get_var(args[i].get_method_name().unwrap()); }
        while args[i].is_application() && args[i].get_application().len() == 1 {
            args[i] = args[i].get_application()[0].clone();
        }
    }
}

pub fn unpack(mut arg: Argument) -> Argument {
    if is_var(&arg) { return unpack(get_var(arg.get_method_name().unwrap())); }
    while arg.is_application() && arg.get_application().len() == 1 {
        arg = arg.get_application()[0].clone();
    }
    arg
}

pub fn eval_args(args: &mut VecDeque<Argument>, len: usize) {
    for i in 0..(if len > 0 && len <= args.len() { len } else { args.len() }) {
        while args[i].is_application() {
            match apply(&mut args[i]) {
                Some(s) => { 
                    if args[i] == s { break; }
                    args[i] = s;
                } , None => { return; }
            };
        }
    }
}

pub fn apply(app: &mut Argument) -> Option<Argument> {
    apply_with(app, unsafe{&LOCALVARS})
}
pub fn apply_with(app: &mut Argument, vars: &Option<Vec<(String, Argument)>>) -> Option<Argument> {
    if !app.is_application() {
        println!("Unexpected call of apply without Application");
        return None;
    }
    if vars.is_some() { unsafe { LOCALVARS=vars.clone(); } }
    let varsbkp = unsafe { LOCALVARS.clone() };
    //println!("{}", app.to_string());
    let mut args = app.get_application();
    if args.len() <= 1 || !args[1].is_operator() { unpack_args(&mut args, 2); }
    if args.len() == 1 && args[0].is_application() { return apply(&mut args[0]); }
    if is_var(&args[0]) || is_var_local(&args[0], &vars) {
        let mut t = args.clone();
        let v = t.pop_front().unwrap().get_method_name().unwrap();
        let mut res = VecDeque::from(vec![if is_var(&Argument::Method(v.clone())) { get_var(v) } else { get_var_local(v, &vars) }]);
        res.append(&mut t);
        return apply(&mut Argument::Application(res));
    }
    if args.len() == 1 && !args[0].is_method() { return Some(args[0].clone()); }
    if args.is_empty() { return None; }
    if args[0].is_application() && (args.len() <= 1 || !args[1].is_operator()) {
        args = {
            let mut t = args.pop_front().unwrap().get_application();
            t.append(&mut args);
            t
        };
    }
    let mut command = if args.len() > 1 && args[1].is_operator() {
        args[1].clone()
    } else {
        args[0].clone()
    };
    if command == Argument::Method("help".to_string()) {
        if args.len() == 1 {
            unsafe {
                for &(ref c, m) in COMMANDS.as_ref().unwrap().iter() {
                    if c.is_method() {
                        print!("{} ", c.to_string());
                    }
                }
            }
            println!("");
            return None;
        } else {
            command = args[1].clone();
            args[1] = Argument::Method("help".to_string());
        }
    }
    unsafe {
        if command.is_method() {
            if is_var(&command) { return Some(get_var(command.get_method_name().unwrap())); }
        }
        for &(ref c, m) in COMMANDS.as_ref().unwrap().iter() {
            if command == *c {
                match m(args) {
                    Err(msg) => { println!("Error: {}", msg); return None; },
                    Ok(res) => {
                        if res.len() == 1 {
                            unsafe { LOCALVARS = varsbkp; }
                            return Some(res[0].clone());
                        }
                        unsafe { LOCALVARS = varsbkp; }
                        return Some(if res.is_empty() { Argument::Nothing } else { Argument::Application(res) });
                    }
                }
            }
        }
    }
    println!("Unknown command: {}", command.to_string());
    return None;
}
