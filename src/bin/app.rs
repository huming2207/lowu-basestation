#![no_main]
#![no_std]

use lowu as _; // global logger + panicking-behavior + memory layout

#[rtic::app(
    device = stm32wlxx_hal::pac,
)]
mod app {
    use bbqueue::{Consumer, Producer, BBBuffer};
    use lowu::mono::MonoTimer;
    use stm32wlxx_hal::{pac, uart::{LpUart, Clk}, gpio::{PortA, pins}};

    // Shared resources go here
    #[shared]
    struct Shared {
        uart: pac::LPUART,
    }

    // Local resources go here
    #[local]
    struct Local {
        uart_bb_cons: Consumer<'static, 1024>,
        uart_bb_prod: Producer<'static, 1024>,
    }

    #[monotonic(binds = LPTIM1, default = true)]
    type LPMonotonic = MonoTimer<pac::LPTIM1>;

    #[init(local = [uart_bbq: BBBuffer<1024> = BBBuffer::new()])]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("init");

        let mut dp = stm32wlxx_hal::pac::Peripherals::take().unwrap();
        let cs = unsafe { &cortex_m::interrupt::CriticalSection::new() };
        unsafe {
            stm32wlxx_hal::rcc::set_sysclk_msi(
                &mut dp.FLASH,
                &mut dp.PWR,
                &mut dp.RCC,
                stm32wlxx_hal::rcc::MsiRange::Range48M,
                cs,
            )
        };
        
        // HSI enable
        dp.RCC.cr.modify(|_, w| w.hsion().set_bit());
        while dp.RCC.cr.read().hsirdy().is_not_ready() {}

        // LSE enable
        dp.RCC.bdcr.modify(|_, w| w.lseon().set_bit());
        while dp.RCC.bdcr.read().lserdy().is_not_ready() {}

        // LSE -> LPTIM enable
        dp.RCC.bdcr.modify(|_, w| w.lsesysen().set_bit());
        while dp.RCC.bdcr.read().lsesysrdy().is_not_ready() {}
    
        // Force DBG up when sleep, should be disabled in release mode
        dp.DBGMCU.cr.modify(|_, w| {
            w.dbg_sleep().set_bit();
            w.dbg_standby().set_bit();
            w.dbg_stop().set_bit()
        });
        dp.RCC.ahb1enr.write(|w| w.dma1en().set_bit());
    
        // Monotonic timer init
        let monotonic: MonoTimer<pac::LPTIM1> = MonoTimer::<pac::LPTIM1>::new(dp.LPTIM1);

        // UART buffer
        let (uart_bb_prod, uart_bb_cons) = cx.local.uart_bbq.try_split().unwrap();

        let gpioa: PortA = PortA::split(dp.GPIOA, &mut dp.RCC);
        let lpuart: LpUart<pins::A3, pins::A2> = LpUart::new(dp.LPUART, 230400, Clk::Lse, &mut dp.RCC)
            .enable_rx(gpioa.a3, cs)
            .enable_tx(gpioa.a2, cs);

        let uart_p = lpuart.free();

        (
            Shared {
                uart: uart_p,
            },
            Local {
                uart_bb_prod,
                uart_bb_cons,
            },
            init::Monotonics(monotonic),
        )
    }

    #[task(binds = LPUART1, local = [uart_bb_prod, uart_bb_cons])]
    fn lpuart_task(cx: lpuart_task::Context) {
        
    }

    // Optional idle, can be removed if not needed.
    #[idle]
    fn idle(_: idle::Context) -> ! {
        defmt::info!("idle");

        loop {
            cortex_m::asm::wfi();
        }
    }
}
