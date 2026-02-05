use std::{thread::sleep, time::Duration};

use sunpower_cryotelgt::{CoolerState, CryoTelGt, StopMode};
use instrumentrs::TcpIpInterface;

use measurements::Temperature;

fn main() {
    let ip_addr = "192.168.1.2:4003";

    // Get our serial instrument interface
    let interface = TcpIpInterface::simple(ip_addr).unwrap();

    // Now we can open the Lakeshore336 with the serial interface.
    let mut inst = CryoTelGt::try_new(interface).unwrap();

    println!("Temp set point: {:?}", inst.get_temperature_setpoint().unwrap());
    // get the current stop mode
    println!("Stop mode: {:?}", inst.get_stop_mode().unwrap());
    println!("Current state: {:?}", inst.get_state().unwrap());

    // get full state
    // println!("Full state: {:?}", inst.get_full_state().unwrap());

    // let set_temp = Temperature::from_kelvin(110.0);
    // inst.set_temperature_setpoint(set_temp).unwrap();
    // inst.set_stop_mode(StopMode::DigitalInput).unwrap();



    for _ in 0..1800 {
        let current_temp = inst.get_temperature().unwrap();
        let current_power_lims = inst.get_power_limits_current().unwrap();
        println!("Current temp: {}, Max power: {}, Min power: {}, Current power: {}", current_temp, current_power_lims.0, current_power_lims.1, current_power_lims.2);
        sleep(Duration::from_secs(5));
    }
        

    // FIXME: Need to try this order
    // inst.set_state(CoolerState::Disabled).unwrap();
    // inst.set_stop_mode(StopMode::Remote).unwrap();
}
