#![feature(async_await, await_macro)]

use upnp::device::{Device, DeviceSpec};
use upnp::scpd::{SCPD, Action, StateVariable};
use upnp::Error;

fn main() -> Result<(), Error> {
    let mut rt = tokio::runtime::current_thread::Runtime::new().unwrap();

    let uri: hyper::Uri = "http://192.168.2.49:1400/xml/device_description.xml"
        .parse()
        .unwrap();

    let device = rt.block_on(Device::from_url(uri))?;
    print(&device);

    Ok(())
}

fn print(device: &Device) {
    print_inner(device.description(), device.ip(), 0);
}

fn print_inner(spec: &DeviceSpec, ip: &hyper::Uri, indent_lvl: usize) {
    let space = "  ".repeat(indent_lvl);

    let device_name = nth_last_colonseparated(spec.device_type(), 1).unwrap().to_uppercase();
    println!("{} {}", space, device_name);

    for service in spec.services() {
        let svc_name = nth_last_colonseparated(service.service_id(), 0).unwrap();
        println!("{} - {}", space, svc_name);

        let fut = SCPD::from_url(service.scpd_url(ip.clone()), service.service_type().to_string());
        let mut rt = tokio::runtime::current_thread::Runtime::new().unwrap();
        let scpd = rt.block_on(fut).unwrap();
        /*for state_var in scpd.state_variables() {
            print_state_var(indent_lvl+2, state_var);
        }
        for action in scpd.actions() {
            print_action(indent_lvl+2, action);
        }*/
    }

    for device in spec.devices() {
        print_inner(device, ip, indent_lvl+1);
    }
}

fn print_action(indent_lvl: usize, action: &Action) {
    let space = "  ".repeat(indent_lvl);

    let inputs: Vec<&str> = action.input_arguments()
        .map(upnp::scpd::Argument::related_state_variable)
        .collect();
    let outputs: Vec<&str> = action.output_arguments()
        .map(upnp::scpd::Argument::related_state_variable)
        .collect();
    print!("{}AC: {}: ({})", space, action.name(), inputs.join(", "));
    if outputs.len() > 0 {
        print!(" -> ({})", outputs.join(", "));
    }
    println!();
}

fn print_state_var(indent_lvl: usize, state_var: &StateVariable) {
    let space = "  ".repeat(indent_lvl);
    println!("{}SV {:?}", space, state_var);
}

fn nth_last_colonseparated(s: &str, mut n: usize) -> Option<&str> {
    let mut iter = s.rsplit(':');
    while n > 0 {
        n -= 1;
        iter.next();
    }
    iter.next()
}
