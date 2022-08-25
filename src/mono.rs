use core::convert::TryInto;

use cortex_m::prelude::_embedded_hal_timer_CountDown;
use fugit;
use stm32wlxx_hal::{
    pac::{RCC, LPTIM1, Peripherals},
    lptim::{self, LpTim, LpTim1},
    rcc,
};


use rtic_monotonic::Monotonic;

pub struct MonoTimer<T, const FREQ: u32> {
    timer: T,

}

impl<const FREQ: u32> MonoTimer<LpTim1, FREQ> {
    pub fn new() -> Self {
        let mut dp: Peripherals = Peripherals::take().unwrap();
        dp.RCC.apb1enr1.modify(|_, w| w.lptim1en().set_bit());
        dp.RCC.apb1rstr1.modify(|_, w| w.lptim1rst().set_bit());
        dp.RCC.apb1rstr1.modify(|_, w| w.lptim1rst().clear_bit());
        
        // LSE = 32768Hz; DIV16 to LPTIM1: 32768 / 16 = 2048Hz
        let mut lptim1: LpTim1 = LpTim1::new(dp.LPTIM1, lptim::Clk::Lse, lptim::Prescaler::Div16, &mut dp.RCC);
        let freq_hz: u32 = lptim1.hz().to_integer(); // Should be 2048 here
        let max_duty: u32 = freq_hz / FREQ;
        
        lptim1.set_max_duty(max_duty.try_into().unwrap());
        Self { timer: lptim1 }
    }
}