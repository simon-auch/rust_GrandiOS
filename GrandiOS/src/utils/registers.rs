//must be inlined because of obvious reasons...
#[inline(always)]
pub fn get_lr() -> u32{
    let lr: u32;
    unsafe{asm!("mov $0, r14":"=r"(lr):::"volatile")}
    lr
}
