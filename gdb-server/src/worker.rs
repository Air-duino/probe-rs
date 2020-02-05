use async_std::prelude::*;
use futures::channel::mpsc;
use futures::future::FutureExt;
use futures::select;
use gdb_protocol::packet::{CheckedPacket, Kind as PacketKind};
use probe_rs::Core;
use probe_rs::Session;
use recap::Recap;
use serde::Deserialize;
use std::sync::{Arc, Mutex};

type ServerResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;

#[allow(clippy::mutex_atomic)]
#[allow(clippy::cognitive_complexity)]
pub async fn worker(
    mut input_stream: Receiver<CheckedPacket>,
    output_stream: Sender<CheckedPacket>,
    session: Arc<Mutex<Session>>,
) -> ServerResult<()> {
    let mut core = session.lock().unwrap().attach_to_core(0).unwrap();
    let mut awaits_halt = false;

    loop {
        select! {
            potential_packet = input_stream.next().fuse() => {
                if let Some(packet) = potential_packet {
                    log::warn!("WORKING {}", String::from_utf8_lossy(&packet.data));
                    if handler(&mut core, output_stream.clone(), &mut awaits_halt, packet).await? {
                        break;
                    }
                } else {
                    break
                }
            },
            _ = await_halt(&core, output_stream.clone(), awaits_halt).fuse() => {}
        }
    }
    Ok(())
}

pub async fn handler(
    core: &mut Core,
    output_stream: Sender<CheckedPacket>,
    awaits_halt: &mut bool,
    packet: CheckedPacket,
) -> ServerResult<bool> {
    let mut break_due = false;
    if packet.is_valid() {
        let packet_string = String::from_utf8_lossy(&packet.data).to_string();
        #[allow(clippy::if_same_then_else)]
        let response: Option<String> = if packet.data.starts_with(b"qSupported") {
            Some("PacketSize=2048;swbreak-;hwbreak+;vContSupported+;qXfer:memory-map:read+".into())
        } else if packet.data.starts_with(b"vMustReplyEmpty") {
            Some("".into())
        } else if packet.data.starts_with(b"qTStatus") {
            Some("".into())
        } else if packet.data.starts_with(b"qTfV") {
            Some("".into())
        } else if packet.data.starts_with(b"qAttached") {
            Some("1".into())
        } else if packet.data.starts_with(b"?") {
            Some("S05".into())
        } else if packet.data.starts_with(b"g") {
            Some("xxxxxxxx".into())
        } else if packet.data.starts_with(b"p") {
            #[derive(Debug, Deserialize, PartialEq, Recap)]
            #[recap(regex = r#"p(?P<reg>\w+)"#)]
            struct P {
                reg: String,
            }

            let p = packet_string.parse::<P>().unwrap();

            let _ = core.halt();
            core.wait_for_core_halted().unwrap();

            let value = core
                .read_core_reg(u8::from_str_radix(&p.reg, 16).unwrap())
                .unwrap();

            format!(
                "{}{}{}{}",
                value as u8,
                (value >> 8) as u8,
                (value >> 16) as u8,
                (value >> 24) as u8
            );

            Some(format!(
                "{:02x}{:02x}{:02x}{:02x}",
                value as u8,
                (value >> 8) as u8,
                (value >> 16) as u8,
                (value >> 24) as u8
            ))
        } else if packet.data.starts_with(b"qTsP") {
            Some("".into())
        } else if packet.data.starts_with(b"qfThreadInfo") {
            Some("".into())
        } else if packet.data.starts_with(b"m") {
            #[derive(Debug, Deserialize, PartialEq, Recap)]
            #[recap(regex = r#"m(?P<addr>\w+),(?P<length>\w+)"#)]
            struct M {
                addr: String,
                length: String,
            }

            let m = packet_string.parse::<M>().unwrap();

            let mut readback_data = vec![0u8; usize::from_str_radix(&m.length, 16).unwrap()];
            core.memory()
                .read_block8(
                    u32::from_str_radix(&m.addr, 16).unwrap(),
                    &mut readback_data,
                )
                .unwrap();

            Some(
                readback_data
                    .iter()
                    .map(|s| format!("{:02x?}", s))
                    .collect::<Vec<String>>()
                    .join(""),
            )
        } else if packet.data.starts_with(b"qL") {
            Some("".into())
        } else if packet.data.starts_with(b"qC") {
            Some("".into())
        } else if packet.data.starts_with(b"qOffsets") {
            Some("".into())
        } else if packet.data.starts_with(b"vCont?") {
            Some("vCont;c;t;s".into())
        } else if packet.data.starts_with(b"vContb;c") || packet.data.starts_with(b"c") {
            core.run().unwrap();
            *awaits_halt = true;
            None
        } else if packet.data.starts_with(b"vContb;t") {
            core.halt().unwrap();
            core.wait_for_core_halted().unwrap();
            *awaits_halt = false;
            Some("OK".into())
        } else if packet.data.starts_with(b"vContb;s") || packet.data.starts_with(b"s") {
            core.step().unwrap();
            *awaits_halt = false;
            Some("S05".into())
        } else if packet.data.starts_with(b"Z0") {
            Some("".into())
        } else if packet.data.starts_with(b"Z1") {
            #[derive(Debug, Deserialize, PartialEq, Recap)]
            #[recap(regex = r#"Z1,(?P<addr>\w+),(?P<size>\w+)"#)]
            struct Z1 {
                addr: String,
                size: String,
            }

            let z1 = packet_string.parse::<Z1>().unwrap();

            let addr = u32::from_str_radix(&z1.addr, 16).unwrap();

            core.reset_and_halt().unwrap();
            core.wait_for_core_halted().unwrap();
            core.set_hw_breakpoint(addr).unwrap();
            core.run().unwrap();
            Some("OK".into())
        } else if packet.data.starts_with(b"z1") {
            #[derive(Debug, Deserialize, PartialEq, Recap)]
            #[recap(regex = r#"z1,(?P<addr>\w+),(?P<size>\w+)"#)]
            struct Z1 {
                addr: String,
                size: String,
            }

            let z1 = packet_string.parse::<Z1>().unwrap();

            let addr = u32::from_str_radix(&z1.addr, 16).unwrap();

            core.reset_and_halt().unwrap();
            core.wait_for_core_halted().unwrap();
            core.clear_hw_breakpoint(addr).unwrap();
            core.run().unwrap();
            Some("OK".into())
        } else if packet.data.starts_with(b"X") {
            #[derive(Debug, Deserialize, PartialEq, Recap)]
            #[recap(regex = r#"X(?P<addr>\w+),(?P<length>\w+):(?P<data>[01]*)"#)]
            struct X {
                addr: String,
                length: String,
                data: String,
            }

            let x = packet_string.parse::<X>().unwrap();

            let length = usize::from_str_radix(&x.length, 16).unwrap();
            let data = &packet.data[packet.data.len() - length..];

            core.memory()
                .write_block8(u32::from_str_radix(&x.addr, 16).unwrap(), data)
                .unwrap();

            Some("OK".into())
        } else if packet.data.starts_with(b"qXfer:memory-mapb:read") {
            let xml = r#"<?xml version="1.0"?>
<!DOCTYPE memory-map PUBLIC "+//IDN gnu.org//DTD GDB Memory Map V1.0//EN" "http://sourceware.org/gdb/gdb-memory-map.dtd">
<memory-map>
<memory type="ram" start="0x20000000" length="0x4000"/>
<memory type="rom" start="0x00000000" length="0x40000"/>
</memory-map>"#;
            Some(
                std::str::from_utf8(&gdb_sanitize_file(xml.as_bytes().to_vec(), 0, 1000))
                    .unwrap()
                    .to_string(),
            )
        } else if packet.data.starts_with(&[0x03]) {
            let _ = core.halt();
            core.wait_for_core_halted().unwrap();
            Some("T05hwbreak:;".into())
        } else if packet.data.starts_with(b"D") {
            break_due = true;
            Some("OK".into())
        } else if packet.data.starts_with(b"qRcmdb,7265736574") {
            let _cpu_info = core.reset();
            let _cpu_info = core.halt();
            Some("OK".into())
        } else if packet.data.starts_with(b"qTfV") {
            Some("".into())
        } else if packet.data.starts_with(b"qTfV") {
            Some("".into())
        } else {
            Some("OK".into())
        };

        if let Some(response) = response {
            let response = CheckedPacket::from_data(PacketKind::Packet, response.into_bytes());

            let mut bytes = Vec::new();
            response.encode(&mut bytes).unwrap();
            log::debug!("{:x?}", std::str::from_utf8(&response.data).unwrap());
            log::debug!("-----------------------------------------------");
            output_stream.unbounded_send(response)?;
        };
    }
    Ok(break_due)
}

pub async fn await_halt(core: &Core, output_stream: Sender<CheckedPacket>, await_halt: bool) {
    if await_halt && core.core_halted().unwrap() {
        let response =
            CheckedPacket::from_data(PacketKind::Packet, "T05hwbreak:;".to_string().into_bytes());

        let mut bytes = Vec::new();
        response.encode(&mut bytes).unwrap();

        let _ = output_stream.unbounded_send(response);
    }
}

fn gdb_sanitize_file(mut data: Vec<u8>, offset: u32, len: u32) -> Vec<u8> {
    let offset = offset as usize;
    let len = len as usize;
    let mut end = offset + len;
    if offset > data.len() {
        b"l".to_vec()
    } else {
        if end > data.len() {
            end = data.len();
        }
        let mut trimmed_data: Vec<u8> = data.drain(offset..end).collect();
        if trimmed_data.len() >= len {
            // XXX should this be <= or < ?
            trimmed_data.insert(0, b'm');
        } else {
            trimmed_data.insert(0, b'l');
        }
        trimmed_data
    }
}
