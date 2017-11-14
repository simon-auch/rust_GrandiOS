use driver::serial::*;
use utils::parser::Argument;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::string::String;
use commands::logo;
use driver::led::*;
use utils::spinlock::*;
use utils::thread::*;

pub fn exec(args: Vec<Argument>) {
    if args.len() == 0 {
        println!("Test what?");
    } else {
        match args[0].get_str().expect("String expected").as_str() {
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
            _ => println!("I don't know that.")
        }
    }
}
