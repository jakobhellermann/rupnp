use async_std::task;
use isahc::http::Uri;
use upnp::device::{Device, DeviceSpec};
use upnp::scpd::{Action, StateVariable, SCPD};
use upnp::Error;

fn main() -> Result<(), upnp::Error> {
    let url: Uri = "http://192.168.2.49:1400/xml/device_description.xml"
        .parse()
        .unwrap();
    task::block_on(dump_scpd(url))
}

async fn dump_scpd(url: Uri) -> Result<(), Error> {
    let device = Device::from_url(url).await?;
    print(&device);

    Ok(())
}

fn print(device: &Device) {
    print_inner(device, device.url(), 0);
}

fn print_inner(spec: &DeviceSpec, url: &Uri, indent_lvl: usize) {
    let space = "  ".repeat(indent_lvl);

    println!("{} {}", space, &spec.device_type);

    for service in spec.services() {
        println!("{} - {}", space, service.service_id());

        let scpd = task::block_on(SCPD::from_url(
            &service.scpd_url(url),
            service.service_type().to_string(),
        ))
        .unwrap();

        for state_var in scpd.state_variables() {
            print_state_var(indent_lvl + 2, state_var);
        }
        for action in scpd.actions() {
            print_action(indent_lvl + 2, action);
        }
    }

    for device in spec.devices() {
        print_inner(device, url, indent_lvl + 1);
    }
}

fn print_action(indent_lvl: usize, action: &Action) {
    let space = "  ".repeat(indent_lvl);

    let inputs: Vec<&str> = action
        .input_arguments()
        .map(upnp::scpd::Argument::related_state_variable)
        .collect();
    let outputs: Vec<&str> = action
        .output_arguments()
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
