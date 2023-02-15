#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

extern crate tinyrlibc;
use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive, AnyPin, Pin};
use embassy_nrf::interrupt;
use embassy_nrf::interrupt::{Priority, InterruptExt};
use embassy_time::{Duration, Timer};
use nrf_modem::{SystemMode, ConnectionPreference, GnssData};
use {defmt_rtt as _, panic_probe as _};
use futures::stream::StreamExt;
// use futures::try_join;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Initializing...");
    let p = embassy_nrf::init(Default::default());

    spawner.spawn(blink(p.P0_02.degrade())).unwrap();

    // let lte_init = init_lte();
    //try_join!(lte_init).unwrap();
    init_lte().await.unwrap();

    let response = nrf_modem::send_at::<64>("AT+CGMI").await.unwrap();

    println!("{:?}", response.as_str());

    let gnss = nrf_modem::Gnss::new().await.unwrap();
    if let Ok(mut stream) = gnss.start_continuous_fix(nrf_modem::GnssConfig::default()) {
        while let Some(Ok(loc)) = stream.next().await {
            match loc {
                GnssData::PositionVelocityTime(p) => info!("lat: {} lon: {}", p.latitude, p.longitude),
                GnssData::Nmea(n) => trace!("{:?}", n.as_str()),
                GnssData::Agps(a) => trace!("AGPS"),
            }
        }
    }

}

async fn init_lte() -> Result<(), nrf_modem::Error> {
    info!("Setting up LTE");
    let egu1 = embassy_nrf::interrupt::take!(EGU1);
    egu1.set_priority(Priority::P4);
    egu1.set_handler(|_| {
        nrf_modem::application_irq_handler();
        cortex_m::asm::sev();
    });
    egu1.enable();

    let ipc = embassy_nrf::interrupt::take!(IPC);
    ipc.set_priority(Priority::P0);
    ipc.set_handler(|_| {
        nrf_modem::ipc_irq_handler();
        cortex_m::asm::sev();
    });
    ipc.enable();

    nrf_modem::init(SystemMode {
        lte_support: true,
        lte_psm_support: true,
        nbiot_support: true,
        gnss_support: true,
        preference: ConnectionPreference::None,
    })
    .await?;
    Ok(())
}

#[embassy_executor::task]
async fn blink(pin: AnyPin) {
    let mut led = Output::new(pin, Level::Low, OutputDrive::Standard);
    info!("Starting Blink");
    loop {
        led.set_high();
        Timer::after(Duration::from_millis(300)).await;
        led.set_low();
        Timer::after(Duration::from_millis(300)).await;
    }
}
