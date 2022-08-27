use fugit;
use paste::paste;
use stm32wlxx_hal::pac;

use rtic_monotonic::Monotonic;

const LSE_FREQ_HZ: u32 = 32768;

pub struct MonoTimer<T> {
    timer: T,
    overflow: u16,
}

macro_rules! create_monotimer_new {
    ($x:expr) => {
        paste! {
            impl MonoTimer<pac::[<LPTIM $x>]> {
                pub fn new(timer: pac::[<LPTIM $x>]) -> Self {
                    let rcc = unsafe { &*pac::RCC::ptr() } ;
                    rcc.ccipr.modify(|_, w| w.lptim1sel().lse()); // Use LSE
                    rcc.apb1enr1.modify(|_, w| w.lptim1en().set_bit()); // Enable LPTIM1
            
                    // Toggle reset
                    rcc.apb1rstr1.modify(|_, w| w.lptim1rst().set_bit()); 
                    rcc.apb1rstr1.modify(|_, w| w.lptim1rst().clear_bit());
            
                    // Enable the compare-match interrupt
                    timer.ier.modify(|_, w| w.cmpmie().set_bit());
            
                    Self { timer, overflow: 0 }
                }
            
                #[inline(always)]
                fn is_overflow(&self) -> bool {
                    self.timer.isr.read().arrm().bit_is_set()
                }
            
                #[inline(always)]
                fn clear_overflow(&self) {
                    self.timer.icr.write(|w| w.arrmcf().set_bit());
                }
            
            }
        }
    };
}

create_monotimer_new!(1);
create_monotimer_new!(2);
create_monotimer_new!(3);

macro_rules! create_monotimer {
    ($x:expr) => {
        paste! {
            impl Monotonic for MonoTimer<pac::[<LPTIM $x>]> {
                const DISABLE_INTERRUPT_ON_EMPTY_QUEUE: bool = false;
                type Instant  = fugit::TimerInstantU32<LSE_FREQ_HZ>;
                type Duration = fugit::TimerDurationU32<LSE_FREQ_HZ>;
            
                #[inline(always)]
                fn now(&mut self) -> Self::Instant {
                    let ctr: u32 = self.timer.cnt.read().cnt().bits().into(); // This is actually only 16-bit - check overflow later
            
                    // Double check if there's any ongoing overflow
                    let overflow = if self.is_overflow() {
                        self.overflow + 1
                    } else {
                        self.overflow
                    } as u32;
                
                    Self::Instant::from_ticks((overflow << 16) + ctr)
                }
            
                fn set_compare(&mut self, instant: Self::Instant) {
                    let now = self.now();
            
                    let compare_register_val = match instant.checked_duration_since(now) {
                        Some(duration) if duration.ticks() > 0xffff => 0,
                        None => 0,
                        Some(_) => instant.duration_since_epoch().ticks() as u16,
                    };
            
                    // Write value to compare register
                    self.timer.cmp.write(|w| w.cmp().bits(compare_register_val));
                }
            
                fn clear_compare_flag(&mut self) {
                    self.timer.icr.write(|w| w.cmpmcf().set_bit());
                }
            
                fn zero() -> Self::Instant {
                    Self::Instant::from_ticks(0)
                }
            
                unsafe fn reset(&mut self) {
                    self.timer.cr.modify(|_, w| w.enable().set_bit());
                    self.timer.arr.write(|w| w.bits(0xffff));
                    self.timer.cr.modify(|_, w| w.cntstrt().set_bit());
                }
            
                fn on_interrupt(&mut self) {
                    if self.is_overflow() {
                        self.clear_overflow();
                        self.overflow += 1;
                    }
                }
            }
        }
    };
}

create_monotimer!(1);
create_monotimer!(2);
create_monotimer!(3);