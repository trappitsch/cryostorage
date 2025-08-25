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
use embassy_time::{Duration, Instant, Ticker, Timer};
use embassy_usb::{Config, UsbDevice};
use postcard_rpc::{
    sender_fmt,
    server::{Dispatch, Sender, Server},
};
use static_cell::StaticCell;

bind_interrupts!(pub struct Irqs {
    USBCTRL_IRQ => usb::InterruptHandler<USB>;
});

use {defmt_rtt as _, panic_probe as _};

pub mod app;
pub mod handlers;

#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: ImageDef = ImageDef::secure_exe();

// Program metadata for `picotool info`.
// This is needed if you are using picotool to flash the device
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"RP2350 template"),
    embassy_rp::binary_info::rp_program_description!(c"An example template for the RP2350"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

fn usb_config(serial: &'static str) -> Config<'static> {
    let mut config = Config::new(0x16c0, 0x27DD);
    config.manufacturer = Some("OneVariable");
    config.product = Some("poststation-pico");
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

    // set up the gate pin for MOSFET and button pin to read the button state
    let btn = Input::new(p.PIN_15, Pull::Up);
    let gate = Output::new(p.PIN_16, Level::Low);
    // set up the pin we read from some other device, e.g., the flow meter status via a mosfet
    let read_pin = Input::new(p.PIN_17, Pull::Up);

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
    let led = Output::new(p.PIN_25, Level::Low);

    let context = app::Context { unique_id, led };

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
    spawner.must_spawn(button_task(btn, gate));

    spawner.must_spawn(input_pin_topic(read_pin));
    info!("Entering loop...");
    // Begin running!
    loop {
        // If the host disconnects, we'll return an error here.
        // If this happens, just wait until the host reconnects
        let _ = server.run().await;
    }
}

/// Upon button press, this turns the gate on for a certain amount of time and then off.
///
/// This is intended for gate valve operations. Debounce is implemented in a very, very ugly way ;)
#[embassy_executor::task]
pub async fn button_task(mut btn: Input<'static>, mut gate: Output<'static>) {
    loop {
        btn.wait_for_low().await;
        info!("Button pressed...");
        gate.set_high();
        Timer::after(Duration::from_millis(100)).await;
        info!("Timer expired");
        gate.set_low();
        Timer::after(Duration::from_millis(1000)).await;  // the ugliest debounce possible...
    }
}

/// Upon button press, this switches the state of the gate.
///
/// This is intended for baking, LED operations. Debounce is implemented in a very, very ugly way ;)
// #[embassy_executor::task]
// pub async fn button_task(mut btn: Input<'static>, mut gate: Output<'static>) {
//     let mut next_gate_high = true;
//    loop {
//         btn.wait_for_low().await;
//         if next_gate_high {
//             gate.set_high();
//             info!("Gate set to high.");
//         } else {
//             gate.set_low();
//             info!("Gate set to low.");
//         }
//         next_gate_high = !next_gate_high;
//         Timer::after(Duration::from_millis(1000)).await; // the ugliest debounce possible...
//     }
// }

/// This is the topic that we send to from postcard-rpc to broadcast status of input
///
/// This will stream to <EndpInputPin> with an `InpPinState` message.
#[embassy_executor::task]
pub async fn input_pin_topic(mut inp_pin: Input<'static>) {
    loop {
        inp_pin.wait_for_any_edge().await;
        Timer::after(Duration::from_millis(50)).await;  // ugly debounce
        let msg = match inp_pin.get_level() {
            Level::Low => "Input pin is LOW",
            Level::High => "Input pin is HIGH",
        };
        info!("{}", msg);
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
