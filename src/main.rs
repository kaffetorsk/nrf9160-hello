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
use embassy_nrf::nvmc::Nvmc;
use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
// use futures::try_join;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Initializing...");
    let p = embassy_nrf::init(Default::default());

    spawner.spawn(blink(p.P0_02.degrade())).unwrap();

    // let lte_init = init_lte();
    //try_join!(lte_init).unwrap();
    init_lte().await.unwrap();

    info!("Hello NVMC!");

    // probe-run breaks without this, I'm not sure why.
    //Timer::after(Duration::from_secs(1)).await;

    let mut f = Nvmc::new(p.NVMC);
    const ADDR: u32 = 0x80000;

    info!("Reading...");
    let mut buf = [0u8; 4];
    unwrap!(f.read(ADDR, &mut buf));
    info!("Read: {=[u8]:x}", buf);

    info!("Erasing...");
    unwrap!(f.erase(ADDR, ADDR + 4096));

    info!("Reading...");
    let mut buf = [0u8; 4];
    unwrap!(f.read(ADDR, &mut buf));
    info!("Read: {=[u8]:x}", buf);

    info!("Writing...");
    unwrap!(f.write(ADDR, &[1, 2, 3, 4]));

    info!("Reading...");
    let mut buf = [0u8; 4];
    unwrap!(f.read(ADDR, &mut buf));
    info!("Read: {=[u8]:x}", buf);

    let response = nrf_modem::send_at::<64>("AT+CGMI").await.unwrap();

    println!("{:?}", response.as_str());

    let gnss = nrf_modem::Gnss::new().await.unwrap();
    if let Ok(mut stream) = gnss.start_continuous_fix(nrf_modem::GnssConfig::default()) {
        while let Some(Ok(loc)) = stream.next().await {
            match loc {
                //GnssData::PositionVelocityTime(p) => info!("lat: {} lon: {}", p.latitude, p.longitude),
                GnssData::PositionVelocityTime(p) => info!(
                    "{{
                        \"coordinates\": [{}, {}]
                        \"speed\": {},
                        \"heading\": {},
                        \"accuracy\": {},
                        \"lastfix\": {}-{}-{}T{}:{}:{}.{},
                    }}",
                    p.latitude, p.longitude, p.speed,
                    p.heading, p.accuracy, p.datetime.year,
                    p.datetime.month, p.datetime.day, p.datetime.hour,
                    p.datetime.minute, p.datetime.seconds, p.datetime.ms
                ),
                _ => ()
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
