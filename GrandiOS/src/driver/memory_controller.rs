//Dokumentation: S.239

use core::ptr::{write_volatile, read_volatile};

pub const MC_BASE_ADRESS : u32 = 0xFFFFFF00;

//lots of consts for all the register bits
const RCR_RCB: u32 = 1 << 0;

#[repr(C)]
struct MemoryConrollerMemoryMap{
	rcr: u32,		//remap controlregister
	asr: u32,		//abort status register
	aasr: u32,		//abort address status register
	mpr: u32,		//master priority register
	reserved_0: [u8; 0x5C-0x10],
	//ebi configuration registers
}

pub struct MemoryController{
	mc: *mut MemoryConrollerMemoryMap,
}

unsafe impl Send for MemoryController { }

impl MemoryController {
    //Marked unsafe because is only safe assuming the base_adress is correct
    pub unsafe fn new(base_adress: u32) -> Self{
        MemoryController{
            mc: base_adress as *mut MemoryConrollerMemoryMap,
        }
    }
    pub fn remap(&mut self){
		unsafe{
		    write_volatile(&mut (*(self.mc)).rcr, RCR_RCB);
        }
	}
}
