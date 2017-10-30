use utils::spinlock;
use core::ptr::{write_volatile, read_volatile};

struct Writer{
	
}

pub const DUMM_BASE_ADRESS : u32 = 0xFFFFF200;

//lots of consts for all the register bits
const CR_RSTRX: u32 = 1 << 2;
const CR_RSTTX: u32 = 1 << 3;
const CR_RXEN:  u32 = 1 << 4;
const CR_RXDIS: u32 = 1 << 5;
const CR_TXEN:  u32 = 1 << 6;
const CR_TXDIS: u32 = 1 << 7;
const CR_RSTSTA:u32 = 1 << 8;

const SR_RXRDY:   u32 = 1 << 0;
const SR_TXRDY:   u32 = 1 << 1;
const SR_ENDRX:   u32 = 1 << 3;
const SR_ENDTX:   u32 = 1 << 4;
const SR_OVRE:    u32 = 1 << 5;
const SR_FRAME:   u32 = 1 << 6;
const SR_PARE:    u32 = 1 << 7;
const SR_TXEMPTY: u32 = 1 << 9;
const SR_TXBUFE:  u32 = 1 << 11;
const SR_RXBUFF:  u32 = 1 << 12;
const SR_COMMTX:  u32 = 1 << 30;
const SR_COMMRX:  u32 = 1 << 31;

#[repr(C)]
struct DebugUnitMemoryMap{
	cr:  u32,	//control register
	mr:  u32,	//mode register
	ier: u32,	//interrupt enable reister
	idr: u32,	//interrupt disable register
	imr: u32,	//interrupt mask register
	sr:  u32,	//status register
	rhr: u8,	//receive holding register
	reserved_0: [u8; 3],
	thr: u8,	//holding register
	reserved_1: [u8; 3],
	brgr:u32,	//baud rate generator
	reserved_2: [u32; 3],
	cidr:u32,	//chip id register
	exid:u32,	//chip id extension register
}

struct DebugUnit{
	dumm: *mut DebugUnitMemoryMap,
}


impl DebugUnit {
	//Marked unsafe because self.on is only safe assuming the base_adress and pin are correct
	pub unsafe fn new(base_adress: u32) -> Self{
		DebugUnit{
			dumm: base_adress as *mut DebugUnitMemoryMap,
		}
	}
	fn transmitter_enable(&mut self) {
		unsafe{
		write_volatile(&mut (*(self.dumm)).cr, CR_TXEN);
		}
	}
	fn transmitter_disable(&mut self) {
		unsafe{
		write_volatile(&mut (*(self.dumm)).cr, CR_TXDIS);
		}
	}
	fn transmitter_reset(&mut self) {
		unsafe{
		write_volatile(&mut (*(self.dumm)).cr, CR_RSTTX);
		}
	}
	fn transmitter_write_character(&mut self, c: u8) {
		unsafe{
		//make sure the last character has been written or moved to the shift register
		while (read_volatile(&mut (*(self.dumm)).sr) & (SR_TXRDY)) == 0 {}
		//write new character
		write_volatile(&mut (*(self.dumm)).thr, c);
		}
	}
}
pub fn print(){
	//do something!
	let test = "Hello world!";
	let mut debug_unit = unsafe { DebugUnit::new(DUMM_BASE_ADRESS) } ;
	debug_unit.transmitter_enable();
	//debug_unit.transmitter_write_character(65u8);
	for c in test.chars(){
		debug_unit.transmitter_write_character(c as u8);
	}
}
