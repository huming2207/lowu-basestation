#![no_main]
#![no_std]

use lowu as _; // global logger + panicking-behavior + memory layout

#[rtic::app(
    device = stm32wlxx_hal::pac,
)]
mod app {
    use lowu::mono::MonoTimer;
    use stm32wlxx_hal::pac;

    // Shared resources go here
    #[shared]
    struct Shared {
        // TODO: Add resources
    }

    // Local resources go here
    #[local]
    struct Local {
        // TODO: Add resources
    }

    #[monotonic(binds = LPTIM1, default = true)]
    type LPMonotonic = MonoTimer<pac::LPTIM1>;

    #[init]
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
    

        let monotonic: MonoTimer<pac::LPTIM1> = MonoTimer::<pac::LPTIM1>::new(dp.LPTIM1);

        // Setup the monotonic timer
        (
            Shared {
                // Initialization of shared resources go here
            },
            Local {
                // Initialization of local resources go here
            },
            init::Monotonics(monotonic),
        )
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
