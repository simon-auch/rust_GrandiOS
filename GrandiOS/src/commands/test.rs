use driver::serial::*;
use utils::parser::Argument;
use core::result::Result;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::string::{String,ToString};
use commands::logo;
use driver::led::*;
use utils::spinlock::*;
use utils::thread::*;

pub fn exec(args: Vec<Argument>) -> Result<Vec<Argument>, String> {
    if args.len() == 0 { return Err("Test what?".to_string()); }
    if !args[0].is_str() { return Err("String expected".to_string()); }
    match args[0].get_str().unwrap().as_str() {
        "size" => {
            let (w, h) = logo::resize();
            println!("{}x{}",w,h);
        },
        "alloc" => {
            {
                let a = Box::new("Hallo");
                let b = Box::new("Welt!");
                println!("{} at {:p}", a, a);
                println!("{} at {:p}", b, b);
            }
            let a = Box::new("Test");
            println!("{} at {:p}", a, a);
        },
        "lock" => {
            let mut led_yellow = unsafe { PIO::new(PIO_LED_YELLOW) };
            let mut led_red    = unsafe { PIO::new(PIO_LED_RED)    };
            let mut led_green  = unsafe { PIO::new(PIO_LED_GREEN)  };
            let lock = Spinlock::new(0u32);
            {
                //lock is hold until data goes out of scope
                let mut data = lock.lock();
                *data += 1;

                    led_yellow.on();
                    let mut data2 = lock.try_lock();
                    match data2{
                        Some(guard) => {
                            //we got the lock, but it should have been locked already..............
                            led_red.on();},
                        None => {
                            led_green.on();},
                    }
                }
            },
            "tcb" => {
                {
                    //TCB again
                    // Take a fn-pointer, make it a rawpointer
                    let idle_thread_function_ptr: *mut _ = idle_thread as *mut _;
                    // Box it
                    let idle = Box::new(idle_thread_function_ptr);
                    // Shove it into the TCB
                    let mut tcb = TCB::new("Test TCB",idle);
                    println!("[{1}] -- {0:?}: {2}", tcb.update_state(), tcb.id, tcb.name);
                    //println!("pc...? {:p}",tcb.program_counter);
                    //tcb.save_registers();
                    //println!("pc...? {:p}",tcb.program_counter);
                    tcb.load_registers();
                    //println!("pc...? {:p}",tcb.program_counter);
                }
            }
        },
        "tcb" => {
            {// TCBs
                let mut t1 = TCB::new(1,"Erster TCB");
                let mut t2 = TCB::new(2,"Zweiter TCB");
                t1.get_state();
                
                println!("[{1}] -- {0:?}: {2}", t1.update_state(), t1.id, t1.name);
                println!("[{1}] -- {0:?}: {2}", t2.update_state(), t2.id, t2.name);
                t2.save_registers();
                t1.load_registers();
            }
        },
        _ => return Err("I don't know that.".to_string())
    }
    Ok(vec![])
}
