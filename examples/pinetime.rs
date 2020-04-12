#![no_std]
#![no_main]

// configure panic behavior:
//use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
use panic_rtt_core::{self, rprintln, rtt_init_print}; // prints panic message to rtt / jlink

use cortex_m_rt as rt;
use nrf52832_hal as p_hal;
use p_hal::nrf52832_pac as pac;
use p_hal::{clocks::ClocksExt, gpio::GpioExt};
use p_hal::{delay::Delay, twim};

use hrs3300_core::HRS3300;
use rt::entry;

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::InputPin;

#[entry]
fn main() -> ! {
    rtt_init_print!(NoBlockTrim);

    let cp = pac::CorePeripherals::take().unwrap();
    let mut delay_source = Delay::new(cp.SYST);

    // PineTime has a 32 MHz HSE (HFXO) and a 32.768 kHz LSE (LFXO)
    let dp = pac::Peripherals::take().unwrap();
    let _clockit = dp.CLOCK.constrain().start_lfclk().enable_ext_hfosc();

    let port0 = dp.P0.split();

    rprintln!("\r\n--- BEGIN ---");

    // P0.12: when this pin is low, it indicates the battery is charging
    let charging_pin = port0.p0_12.into_floating_input();
    // P0.19: power presence? when this line is low, there's power connected
    let power_pin = port0.p0_19.into_floating_input();
    let is_charging = charging_pin.is_low().unwrap_or(false);
    let is_powered = power_pin.is_low().unwrap_or(false);

    rprintln!("power: {} charging: {}", is_powered, is_charging);

    // internal i2c0 bus devices: BMA421 (accel), HRS3300 (hrs), CST816S (TouchPad)
    // BMA421-INT:  P0.08
    // TP-INT: P0.28
    let i2c0_pins = twim::Pins {
        scl: port0.p0_07.into_floating_input().degrade(),
        sda: port0.p0_06.into_floating_input().degrade(),
    };
    let i2c_port = twim::Twim::new(dp.TWIM1, i2c0_pins, twim::Frequency::K400);
    let i2c_bus0 = shared_bus::CortexMBusManager::new(i2c_port);

    let mut hrs = HRS3300::default(i2c_bus0.acquire());
    hrs.init().unwrap();

    // give user some time to stick their wrist under the pinetime
    delay_source.delay_ms(3000u32);

    let mut valid_samples = 0;
    rprintln!("\r\n--- DATA >>> ---");
    for _tick in 0..500 {
        if let Ok((c0data, c1data)) = hrs.read_raw_sample() {
            rprintln!("{}, {}", c0data, c1data);
            valid_samples += 1;
        }
        delay_source.delay_ms(50u8); //50 ms between attempts
    }

    rprintln!("\r\n--- <<< DATA {}---", valid_samples);
    hrs.enable(false).unwrap();

    loop {
        //sleep forever since we have no interrupts setup yet
        cortex_m::asm::wfi();
    }
}
