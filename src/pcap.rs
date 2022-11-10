use std::{
    fs::{File, OpenOptions},
    io::Write,
    mem,
    path::Path,
    slice, thread,
    time::Duration,
};

use if_addrs::Interface;
use pcap::{Capture, Device, Packet};
use uuid::Uuid;
use yansi::Paint;

pub fn capture(interface: &Interface, uuid: &Uuid, pcap: &Option<String>) {
    thread::sleep(Duration::from_secs(2));
    info!(
        "Interface used for SIP {} {}",
        Paint::red(format!("[{}]", interface.name)),
        Paint::yellow(format!("[{}]", interface.addr.ip()))
    );

    if pcap.is_none() {
        info!("Available interfaces for capture");
        for dev in Device::list().unwrap().into_iter() {
            info!(
                "{:?}:{}",
                Paint::red(format!("[{}]", dev.name)),
                dev.desc.unwrap_or_else(|| "Unknown".to_string())
            );
        }
    }

    let device = Device::list().unwrap().into_iter().find(|dev| match pcap {
        Some(name) => dev.name.contains(name),
        None => false,
    });

    write_pcap_header(uuid.to_string());
    match device {
        Some(captured_device) => {
            info!("Capturing now on {:?}", &captured_device.desc);
            let mut cap = Capture::from_device(captured_device)
                .unwrap()
                .promisc(true)
                .snaplen(65535)
                .open()
                .unwrap();

            while let Ok(packet) = cap.next_packet() {
                write_pcap(&packet, uuid.to_string())
            }
        }
        None => info!("could not find a device to start pcap on"),
    };
}

fn write_pcap_header(name: String) {
    if !Path::new(&format!("{}.pcap", name)).exists() {
        File::create(format!("{}.pcap", name)).unwrap();
    }
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(&format!("{}.pcap", name))
        .unwrap();
    unsafe {
        file.write_all(any_as_u8_slice(&(2712847316_u32))).unwrap();
        file.write_all(any_as_u8_slice(&(2_u16))).unwrap();
        file.write_all(any_as_u8_slice(&(4_u16))).unwrap();
        file.write_all(any_as_u8_slice(&(0_i32))).unwrap();
        file.write_all(any_as_u8_slice(&(0_u32))).unwrap();
        file.write_all(any_as_u8_slice(&(65535_u32))).unwrap();
        file.write_all(any_as_u8_slice(&(1_u32))).unwrap();
    }
}

fn write_pcap(packet: &Packet, name: String) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(&format!("{}.pcap", name))
        .unwrap();
    unsafe {
        file.write_all(any_as_u8_slice(&(packet.header.ts.tv_sec as u32)))
            .unwrap();
        file.write_all(any_as_u8_slice(&(packet.header.ts.tv_usec as u32)))
            .unwrap();
        file.write_all(any_as_u8_slice(&packet.header.caplen))
            .unwrap();
        file.write_all(any_as_u8_slice(&packet.header.len)).unwrap();
    }
    file.write_all(packet.data).unwrap();
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    slice::from_raw_parts((p as *const T) as *const u8, mem::size_of::<T>())
}
