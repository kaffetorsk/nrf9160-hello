#![no_main]
#![no_std]

use hal::{gpio, prelude::*, pwm, pwm::Pwm, timer, timer::Timer};
use nb::block;
use nrf9160_hal as hal;
use rtt_target::{rprintln, rtt_init_print};

#[panic_handler] // panicking behavior
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}

#[cortex_m_rt::entry]
fn main() -> ! {
    rtt_init_print!();

    let p = hal::pac::Peripherals::take().unwrap();

    let (pwm, mut timer) = init_device(p);

    pwm.set_period(500u32.hz());

    rprintln!("HELLO World!");

    let wait_time = 1_000_000u32 / pwm.get_max_duty() as u32;
    loop {
        for duty in 0..pwm.get_max_duty() {
            pwm.set_duty_on_common(duty);
            delay(&mut timer, wait_time);
        }
    }
}

fn init_device(p: hal::pac::Peripherals) -> (Pwm<hal::pac::PWM0_NS>, Timer<hal::pac::TIMER0_NS>) {
    let p0 = gpio::p0::Parts::new(p.P0_NS);

    let pwm = Pwm::new(p.PWM0_NS);
    pwm.set_output_pin(
        pwm::Channel::C0,
        p0.p0_02.into_push_pull_output(gpio::Level::High).degrade(),
    );

    let timer = Timer::new(p.TIMER0_NS);

    (pwm, timer)
}

fn delay<T>(timer: &mut Timer<T>, cycles: u32)
where
    T: timer::Instance,
{
    timer.start(cycles);
    let _ = block!(timer.wait());
}
