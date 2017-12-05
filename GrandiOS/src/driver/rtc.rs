//Dokumentation: S.305

use core::ptr::{read_volatile, write_volatile};

pub const RTC_BASE_ADDRESS : u32 = 0xFFFF_FE00;

#[repr(C)]
struct RTCMemoryMap{
	cr: u32,	//control register
	mr: u32,	//mode register
	timr: u32,	//time register
	calr: u32,	//calendar register
	timalr: u32,//time alarm register
	calalr: u32,//calendar alarm register
    sr: u32,    //status register
    sccr: u32,  //status clear command register
    ier: u32,   //interrupt enable register
    idr: u32,   //interrupt disable register
    imr: u32,   //interrupt mask register
    ver: u32,   //valid entry register
}

pub struct RTCController{
	rtc: *mut RTCMemoryMap,
}

impl RTCController {
    //Marked unsafe because is only safe assuming the base_adress is correct
    pub unsafe fn new(base_address: u32) -> Self{
        RTCController{
            rtc: base_address as *mut RTCMemoryMap,
        }
    }
    pub fn interrupt_enable(&mut self) {
		unsafe{ write_volatile(&mut (*(self.rtc)).ier, 1<<2); }
	}
    pub fn interrupt_disable(&mut self) {
		unsafe{ write_volatile(&mut (*(self.rtc)).idr, 1<<2); }
	}
    pub fn interrupt_get(&mut self) -> bool {
		unsafe{ (read_volatile(&mut (*(self.rtc)).imr)&(1<<2))==(1<<2) }
	}
    pub fn has_time_event(&mut self) -> bool {
		unsafe{ (read_volatile(&mut (*(self.rtc)).sr)&(1<<2))==(1<<2) }
    }
}
