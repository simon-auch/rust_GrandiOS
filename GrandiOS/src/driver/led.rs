use core::ptr::{read_volatile,write_volatile};

pub const PIO_B: u32 = 0xfffff600;
pub const PIO_C: u32 = 0xfffff800;

pub const PIO_LED_YELLOW: PIO_PIN = PIO_PIN{ base_adress: PIO_B, pin: 1 << 27 };
pub const PIO_LED_RED:    PIO_PIN = PIO_PIN{ base_adress: PIO_C, pin: 1 <<  0 };
pub const PIO_LED_GREEN:  PIO_PIN = PIO_PIN{ base_adress: PIO_C, pin: 1 <<  1 };

#[allow(non_camel_case_types)]
pub struct PIO_PIN{
	base_adress: u32,
	pin: u32,
}

#[allow(non_camel_case_types)]
#[repr(C)]
struct HW_PIO{
	per: u32,	//pio enable register
	pdr: u32,	//pio disable register
	psr: u32,	//pio status register
	reserver_0: u32,
	oer: u32,	//pio output enable register
	odr: u32,	//pio output disable register
	osr: u32,	//pio output status register
	reserved_1: u32,
	ifer: u32,	//pio glitch input filter enable register
	ifdr: u32,	//pio glitch input filter disable register
	ifsr: u32,	//pio glitch input filter status register
	reserved_2: u32,
	sodr: u32,	//pio set output data register
	codr: u32,	//pio clear output data register
	odsr: u32,	//pio output data status register
}

pub struct PIO{
	hw_pio: *mut HW_PIO,
	pin: u32,
}


impl PIO {
	//Marked unsafe because self.on is only safe assuming the base_adress and pin are correct
	pub unsafe fn new(pio_pin: PIO_PIN) -> Self{
		PIO{
			hw_pio: pio_pin.base_adress as *mut HW_PIO,
			pin: pio_pin.pin,
		}
	}
    pub fn set(&mut self, state: bool) {
        if state {
            self.on();
        } else {
            self.off();
        }
    }
	pub fn on(&mut self){
		unsafe {
		//Initialisieren
		write_volatile(&mut (*(self.hw_pio)).per, self.pin);
		write_volatile(&mut (*(self.hw_pio)).oer, self.pin);
		//Anschalten
		write_volatile(&mut (*(self.hw_pio)).sodr, self.pin);
		}
	}
	pub fn off(&mut self){
		unsafe {
		//Initialisieren
		write_volatile(&mut (*(self.hw_pio)).per, self.pin);
		write_volatile(&mut (*(self.hw_pio)).oer, self.pin);
		//Ausschalten
		write_volatile(&mut (*(self.hw_pio)).codr, self.pin);
		}
	}
    pub fn is_on(&mut self) -> bool {
        unsafe {
		//Initialisieren
		write_volatile(&mut (*(self.hw_pio)).per, self.pin);
		write_volatile(&mut (*(self.hw_pio)).oer, self.pin);
        //Lesen
		read_volatile(&mut (*(self.hw_pio)).odsr);
        (*(self.hw_pio)).odsr & self.pin != 0
        }
    }
}
