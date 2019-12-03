use async_std::task::spawn;
use futures::prelude::*;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::Duration,
};
use upnp::{
    http::Uri,
    ssdp::{SearchTarget, URN},
    DeviceSpec, Error, Service,
};

#[async_std::main]
async fn main() -> Result<(), Error> {
    let mut devices: Vec<_> = upnp::discover(&SearchTarget::RootDevice, Duration::from_secs(1))
        .await?
        .try_collect()
        .await?;

    devices.sort_by_key(|d| d.device_type().clone());
    devices.dedup_by_key(|d| d.device_type().clone());

    let path = PathBuf::from("descriptions");
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }

    for device in devices {
        print(&device, device.url(), 0, &path)?;
        println!("");
    }

    Ok(())
}

fn print(device: &DeviceSpec, url: &Uri, indentation: usize, path: &Path) -> Result<(), Error> {
    let path = path.join(urn_to_str(device.device_type()));
    fs::create_dir_all(&path)?;

    let i = "  ".repeat(indentation);

    println!("{}{}", i, urn_to_str(device.device_type()));

    for service in device.services() {
        let svc = urn_to_str(service.service_type());
        let svc_file = fs::File::create(path.join(&svc))?;

        spawn(
            write_service(svc_file, service.clone(), url.clone()).map(|x| match x {
                Err(e) => eprintln!("failed to fetch and write scpd: {}", e),
                _ => {}
            }),
        );

        println!("{}  - {}", i, svc);
    }

    for device in device.devices() {
        print(device, url, indentation + 1, &path)?;
    }

    Ok(())
}

async fn write_service(mut w: impl Write, service: Service, url: Uri) -> Result<(), Error> {
    let scpd = service.scpd(&url).await?;

    writeln!(w, "StateVars {{")?;
    for state_var in scpd.state_variables() {
        writeln!(w, "  {}", state_var)?;
    }
    writeln!(w, "}}\n\nActions {{")?;
    for action in scpd.actions() {
        writeln!(w, "  {}", action)?;
    }
    writeln!(w, "}}")?;

    Ok(())
}

fn urn_to_str(urn: &URN) -> String {
    urn.typ().to_string().to_lowercase()
}
