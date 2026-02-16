use std::{thread::sleep, time::Duration};

use instrumentrs::{SerialInterface, TcpIpInterface};
use pfeiffer_omnicontrol::{BaseAddress, Omnicontrol, SensorStatus};

fn main() {
    // let interface= SerialInterface::simple("/dev/ttyUSB0", 9600).unwrap();
    let interface = TcpIpInterface::simple("192.168.1.2:4002").unwrap();
    let mut inst = Omnicontrol::new(interface, BaseAddress::Zero);

    // println!("{:?}", inst.get_name());
    //
    let mut ch1 = inst.get_channel(1).unwrap();
    let mut ch2 = inst.get_channel(2).unwrap();

    println!("Pressure Channel 1: {:?} mbar", ch1.get_pressure().unwrap().as_millibars());
    println!("Pressure Channel 2: {:?} mbar", ch2.get_pressure().unwrap().as_millibars());

    println!("Status Channel 1: {:?}", ch1.get_status().unwrap());
    println!("Status Channel 2: {:?}", ch2.get_status().unwrap());


    // ch2.set_status(SensorStatus::Off).unwrap();
    // ch1.set_status(SensorStatus::On).unwrap();
    // ch2.set_status(SensorStatus::On).unwrap();
    // println!("Status Channel 2: {:?}", ch2.get_status().unwrap());
}
