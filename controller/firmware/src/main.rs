#![no_std]
#![no_main]

use app::AppTx;
use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    block::ImageDef,
    gpio::{Input, Level, Output, Pull},
    peripherals::USB,
    usb,
};
use embassy_time::{Duration, Instant, Ticker};
use embassy_usb::{Config, UsbDevice};
use postcard_rpc::{
    sender_fmt,
    server::{Dispatch, Sender, Server},
};
use static_cell::StaticCell;

bind_interrupts!(pub struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
});

use crate::{
    baking::{BakingCtrl, baking_task},
    valve::{ValveStat, valve_pump_task},
};

use {defmt_rtt as _, panic_probe as _};

pub mod app;
pub mod baking;
pub mod handlers;
pub mod valve;

use valve::{ValveCtrl, valve_transfer_task};

const VALVE_PULSE_DURATION_MS: u64 = 200;

#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: ImageDef = ImageDef::secure_exe();

fn usb_config(serial: &'static str) -> Config<'static> {
    let mut config = Config::new(0x16c0, 0x27DD);
    config.manufacturer = Some("Reto Trappitsch, EPFL");
    config.product = Some("Cryostorage Controller");
    config.serial_number = Some(serial);

    // Required for windows compatibility.
    // https://developer.nordicsemi.com/nRF_Connect_SDK/doc/1.9.1/kconfig/CONFIG_CDC_ACM_IAD.html#help
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config.composite_with_iads = true;

    config
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // SYSTEM INIT
    info!("Start");
    let p = embassy_rp::init(Default::default());

    let unique_id: u64 = embassy_rp::otp::get_chipid().unwrap();
    static SERIAL_STRING: StaticCell<[u8; 16]> = StaticCell::new();
    let mut ser_buf = [b' '; 16];
    // This is a simple number-to-hex formatting
    unique_id
        .to_be_bytes()
        .iter()
        .zip(ser_buf.chunks_exact_mut(2))
        .for_each(|(b, chs)| {
            let mut b = *b;
            for c in chs {
                *c = match b >> 4 {
                    v @ 0..10 => b'0' + v,
                    v @ 10..16 => b'A' + (v - 10),
                    _ => b'X',
                };
                b <<= 4;
            }
        });
    let ser_buf = SERIAL_STRING.init(ser_buf);
    let ser_buf = core::str::from_utf8(ser_buf.as_slice()).unwrap();

    // USB/RPC INIT
    let driver = usb::Driver::new(p.USB, Irqs);
    let pbufs = app::PBUFS.take();
    let config = usb_config(ser_buf);

    // PIN CONFIGURATION
    // FIXME: Light switch to PIN_0
    let p_light = Output::new(p.PIN_25, Level::Low);

    let p_baking = Output::new(p.PIN_1, Level::Low);

    // FIXME: Valve position v1 -> v2
    let p_valve_pump_open = Output::new(p.PIN_7, Level::Low);
    let p_valve_pump_close = Output::new(p.PIN_6, Level::Low);
    let p_valve_pump_status_open = Input::new(p.PIN_8, Pull::None);
    let p_valve_pump_status_closed = Input::new(p.PIN_9, Pull::None);

    let p_valve_transfer_open = Output::new(p.PIN_3, Level::Low);
    let p_valve_transfer_close = Output::new(p.PIN_2, Level::Low);
    let p_valve_transfer_status_open = Input::new(p.PIN_4, Pull::None);
    let p_valve_transfer_status_closed = Input::new(p.PIN_5, Pull::None);

    // Baking
    let baking_ctrl = BakingCtrl::default();

    // Valves
    let valve_pump = ValveCtrl::new(
        p_valve_pump_open,
        p_valve_pump_close,
        VALVE_PULSE_DURATION_MS,
    );
    let valve_pump_status = ValveStat::new(p_valve_pump_status_open, p_valve_pump_status_closed);

    let valve_transfer = ValveCtrl::new(
        p_valve_transfer_open,
        p_valve_transfer_close,
        VALVE_PULSE_DURATION_MS,
    );
    let valve_transfer_status =
        ValveStat::new(p_valve_transfer_status_open, p_valve_transfer_status_closed);

    let context = app::Context {
        unique_id,
        p_light,
        baking_ctrl,
        valve_pump_status,
        valve_transfer_status,
    };

    let (device, tx_impl, rx_impl) =
        app::STORAGE.init_poststation(driver, config, pbufs.tx_buf.as_mut_slice());
    let dispatcher = app::MyApp::new(context, spawner.into());
    let vkk = dispatcher.min_key_len();
    let mut server: app::AppServer = Server::new(
        tx_impl,
        rx_impl,
        pbufs.rx_buf.as_mut_slice(),
        dispatcher,
        vkk,
    );
    let sender = server.sender();
    // We need to spawn the USB task so that USB messages are handled by
    // embassy-usb
    spawner.must_spawn(usb_task(device));
    spawner.must_spawn(logging_task(sender));
    spawner.must_spawn(valve_transfer_task(valve_transfer));
    spawner.must_spawn(valve_pump_task(valve_pump));
    spawner.must_spawn(baking_task(p_baking));

    // Begin running!
    loop {
        // If the host disconnects, we'll return an error here.
        // If this happens, just wait until the host reconnects
        let _ = server.run().await;
    }
}

/// This handles the low level USB management
#[embassy_executor::task]
pub async fn usb_task(mut usb: UsbDevice<'static, app::AppDriver>) {
    usb.run().await;
}

/// This task is a "sign of life" logger
#[embassy_executor::task]
pub async fn logging_task(sender: Sender<AppTx>) {
    let mut ticker = Ticker::every(Duration::from_secs(3));
    let start = Instant::now();
    loop {
        ticker.next().await;
        let _ = sender_fmt!(sender, "Uptime: {:?}", start.elapsed()).await;
    }
}
