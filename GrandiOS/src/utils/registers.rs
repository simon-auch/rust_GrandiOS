//must be inlined because of obvious reasons...
#[inline(always)]
pub fn get_lr() -> u32{
    let lr: u32;
    unsafe{asm!("mov r0, r14":"={r0}"(lr):::"volatile")}
    lr
}
