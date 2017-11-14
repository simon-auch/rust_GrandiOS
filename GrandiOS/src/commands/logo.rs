use driver::serial::*;
use utils::parser::Argument;
use utils::shell::*;
use core::result::Result;
use alloc::string::{String,ToString};
use alloc::vec::Vec;

pub fn exec(args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    draw();
    Ok(vec![])
}

pub fn draw() {
    clear();
    let d: Vec<&str> = include_str!("../logo.txt").split('\n').collect();
    let h = d.len() as i32;
    let w = d[0].len() as i32;
    for s in (0..(h/2)).rev() {
        let a = w/2 - s+1;
        let b = h/2 - s+1;
        //Bresenham for ellipses based on code example from wikipedia
        let xm = w/2;
        let ym = h/2;
        let mut dx = 0;
        let mut dy = b;
        let a2 = w*w/4;
        let b2 = h*h/4;
        let mut err = b2-(2*b-1)*a2;
        //Attention! This is actually a do-while-loop!
        while {
            for &(x, y) in [(xm+dx, ym+dy), (xm-dx, ym+dy), (xm-dx, ym-dy), (xm+dx, ym-dy)].iter() {
                draw_logo_at(0, x, y, &d);
            }
            if 2*err < (2*dx+1)*b2 { dx += 1; err += (2*dx+1)*b2; }
            if 2*err > -(2*dy-1)*a2 { dy -= 1; err -= (2*dy-1)*a2; }

            dy >= 0
        } {}

        /*
        while dx < xm {
            dx += 1;
            for &(x, y) in [(xm+dx, ym), (xm-dx, ym)].iter() {
                draw_logo_at(0, x, y, &d);
            }
        }
        */
    }
    draw_name(w);
    print!("{}[{};1H", 27 as char, h-1);
}

fn draw_name(offset: i32) {
    let d: Vec<&str> = include_str!("../name.txt").split('\n').collect();
    let h = d.len() as i32;
    let w = d[(h-2) as usize].len() as i32;
    for i in 0..(h+w) {
        for j in 0..w {
            draw_logo_at(offset, j, i-j, &d);
        }
    }
}

fn draw_logo_at(offset: i32, x: i32, y: i32, d: &Vec<&str>) {
    if y < 0 || x < 0 { return; }
    if y as usize >= d.len() { return; }
    if x as usize >= d[y as usize].as_bytes().len() { return; }
    let c = d[y as usize].as_bytes()[x as usize] as char;
    if c == ' ' { return; }
    for i in 0..1000 {
        unsafe { asm!("nop" :::: "volatile"); }
    }
    draw_at((x+offset) as u32, y as u32+1, c);
}

pub fn draw_at(x: u32, y: u32, c: char) {
    print!("{}[{};{}H{}", 27 as char, y, x, c);
}

pub fn clear() {
    print!("{}[2J", 27 as char);
}

pub fn resize() -> (u32, u32) {
    print!("{}7", 27 as char);
    print!("{}[999:999H", 27 as char);
    let res = get_position();
    print!("{}8", 27 as char);
    res
}
