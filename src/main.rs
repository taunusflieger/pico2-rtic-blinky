#![no_std]
#![no_main]

use rtic_monotonics::rp235x::prelude::*;
// Alias for our HAL crate
use rp235x_hal as hal;

rp235x_timer_monotonic!(Mono);

// Tell the Boot ROM about our application
#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

/// Program metadata for `picotool info`
#[link_section = ".bi_entries"]
#[used]
pub static PICOTOOL_ENTRIES: [hal::binary_info::EntryAddr; 5] = [
    hal::binary_info::rp_cargo_bin_name!(),
    hal::binary_info::rp_cargo_version!(),
    hal::binary_info::rp_program_description!(c"Blinky Example"),
    hal::binary_info::rp_cargo_homepage_url!(),
    hal::binary_info::rp_program_build_attribute!(),
];

/// External high-speed crystal on the Raspberry Pi Pico 2 board is 12 MHz.
/// Adjust if your board has a different frequency
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

#[rtic::app(device = rp235x_hal::pac)]
mod app {
    use super::*;

    use embedded_hal::digital::OutputPin;
    use embedded_hal_0_2::digital::v2::ToggleableOutputPin;
    use panic_probe as _;
    use rp235x_hal::{
        clocks,
        gpio::{self, bank0::Gpio25, FunctionSio, PullDown, SioOutput},
        sio::Sio,
        watchdog::Watchdog,
    };

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: gpio::Pin<Gpio25, FunctionSio<SioOutput>, PullDown>,
    }

    #[init()]
    fn init(mut ctx: init::Context) -> (Shared, Local) {
        // Configure the clocks, watchdog - The default is to generate a 125 MHz system clock
        Mono::start(ctx.device.TIMER0, &mut ctx.device.RESETS); // default rp2040 clock-rate is 125MHz
        let mut watchdog = Watchdog::new(ctx.device.WATCHDOG);
        let _ = clocks::init_clocks_and_plls(
            XTAL_FREQ_HZ,
            ctx.device.XOSC,
            ctx.device.CLOCKS,
            ctx.device.PLL_SYS,
            ctx.device.PLL_USB,
            &mut ctx.device.RESETS,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        // Init LED pin
        let sio = Sio::new(ctx.device.SIO);
        let gpioa = hal::gpio::Pins::new(
            ctx.device.IO_BANK0,
            ctx.device.PADS_BANK0,
            sio.gpio_bank0,
            &mut ctx.device.RESETS,
        );
        let mut led = gpioa.gpio25.into_push_pull_output();
        led.set_low().unwrap();

        // Spawn heartbeat task
        heartbeat::spawn().ok();

        // Return resources and timer
        (Shared {}, Local { led })
    }

    #[task(local = [led])]
    async fn heartbeat(ctx: heartbeat::Context) {
        // Loop forever.
        //
        // It is important to remember that tasks that loop
        // forever should have an `await` somewhere in that loop.
        //
        // Without the await, the task will never yield back to
        // the async executor, which means that no other lower or
        // equal  priority task will be able to run.
        loop {
            // Flicker the built-in LED
            _ = ctx.local.led.toggle();

            Mono::delay(250.millis()).await;
        }
    }
}
