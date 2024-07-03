#![feature(type_alias_impl_trait, impl_trait_in_assoc_type)]
#![no_main]
#![no_std]

use defmt::{info, unwrap};
use embassy_executor::Spawner;

use defmt_rtt as _;
use embassy_futures::select::{select, Either};
use embassy_futures::yield_now;
use embassy_nrf::bind_interrupts;
use embassy_nrf::gpio::Input;
use embassy_nrf::peripherals::RADIO;
use embassy_nrf::radio::{
    ble::{Mode, Radio, TxPower},
    InterruptHandler,
};
use panic_probe as _;

bind_interrupts!(struct Irqs {
    RADIO => InterruptHandler<RADIO>;
});

const ADV_ADDRESS: u32 = 0x8E_89_BE_D6;
const ADV_CRC_INIT: u32 = 0x555555;
const CRC_POLY: u32 = 0b00000001_00000000_00000110_01011011;

#[embassy_executor::main]
async fn main(_s: Spawner) {
    let mut config = embassy_nrf::config::Config::default();
    config.hfclk_source = embassy_nrf::config::HfclkSource::ExternalXtal;
    let mut p = embassy_nrf::init(config);
    let mut start = Input::new(p.P0_11, embassy_nrf::gpio::Pull::Up);
    let mut stop = Input::new(p.P0_12, embassy_nrf::gpio::Pull::Up);
    let data = [0xaa; 255];

    loop {
        info!("Press button 1 start jamming, and button 2 to stop jamming");
        start.wait_for_rising_edge().await;
        info!("Starting jamming all channels");
        match select(stop.wait_for_low(), async {
            let mut radio = Radio::new(&mut p.RADIO, Irqs);
            loop {
                for freq in (2402..2482).step_by(2) {
                    radio.set_tx_power(TxPower::POS8D_BM);
                    radio.set_header_expansion(false);
                    radio.set_access_address(ADV_ADDRESS);
                    radio.set_crc_init(ADV_CRC_INIT);
                    radio.set_crc_poly(CRC_POLY);
                    radio.set_frequency(freq);
                    radio.set_mode(Mode::BLE_LR125KBIT);
                    unwrap!(radio.transmit(&data).await);
                    yield_now().await;
                }
            }
        })
        .await
        {
            Either::First(_) => {
                info!("Jamming stopped");
            }
            Either::Second(_) => {}
        }
    }
}
