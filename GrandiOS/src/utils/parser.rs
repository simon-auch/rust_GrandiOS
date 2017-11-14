use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::string::String;

pub enum Argument {
    Int(usize), Str(String), List(Box<Argument>)
}

impl Argument {
    pub fn get_str(&self) -> Option<String> {
        match self {
            &Argument::Str(ref s) => Some(s.clone()),
            _ => None
        }
    }
    pub fn get_int(&self) -> Option<usize> {
        match self {
            &Argument::Int(i) => Some(i),
            _ => None
        }
    }
}

pub fn parse(mut args: Vec<&str>) -> Vec<Argument> {
    let mut result = vec![];
    while args.len() > 0 {
        let c = args.remove(0).as_bytes();
        if c.len() == 0 { continue; }
        if (48..58).contains(c[0]) { //number ahead
            let mut p = 0;
            let mut r = 0;
            let mut base = 10;
            if c.len() > 1 && c[1] == 120 { //we found 0x
                base = 16;
                p += 2;
            }
            while p < c.len() {
                let mut v = (c[p] as usize) - 48;
                if v > 9 { v = v - 39; }
                if v < 0 { v = v + 32; }
                r = r*base+v;
                p += 1;
            }
            result.push(Argument::Int(r));
            continue;
        }
        result.push(Argument::Str(String::from_utf8(c.to_vec()).unwrap()));
    }
    result
}
