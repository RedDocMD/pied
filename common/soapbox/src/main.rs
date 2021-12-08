use std::{
    env,
    fs::File,
    io::Read,
    path::Path,
    process,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use colored::*;
use error::SoapboxResult;
use indicatif::ProgressIterator;
use serial::Serial;

use crate::error::SoapboxError;

mod console;
mod error;
mod serial;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: {} <serial-port> <input-file>", args[0]);
        process::exit(1);
    }
    soapbox(&args);
}

fn soapbox(args: &[String]) -> SoapboxResult<()> {
    let mut serial = open_serial(&args[1]);
    wait_for_payload_request(&mut serial)?;
    let data = load_payload(&args[1])?;
    send_size(&mut serial, data.len())?;
    send_payload(&mut serial, &data)?;
    terminal(serial);
    Ok(())
}

// fn unexpected_error(err: &anyhow::Error) -> ! {
//     println!(
//         "\n[{}] ⚡ {}",
//         SHORT_NAME,
//         format!("Unexpected Error: {}", err).bright_red()
//     );
//     process::exit(1);
// }

const SHORT_NAME: &str = "SB";

fn wait_for_serial(tty_name: &str) {
    if !serial::is_serial_connected(tty_name) {
        println!("[{}] ⏳ Waiting for {}", SHORT_NAME, tty_name);
        while !serial::is_serial_connected(tty_name) {
            thread::sleep(Duration::from_secs(1));
        }
    }
}

fn open_serial(tty_name: &str) -> Serial {
    wait_for_serial(tty_name);
    match Serial::new(tty_name) {
        Ok(serial) => {
            println!("[{}] ✅ Serial connected", SHORT_NAME);
            serial
        }
        Err(err) => {
            println!("[{}] 🚫 Failed to connect - {}", SHORT_NAME, err);
            process::exit(1);
        }
    }
}

fn wait_for_payload_request(serial: &mut Serial) -> SoapboxResult<()> {
    println!("[{}] 🔌 Please power the target now", SHORT_NAME);
    let mut buf = [0_u8; 4096];
    serial.read(&mut buf)?;
    let start_time = Instant::now();
    let timeout_duration = Duration::from_secs(10);
    let mut count = 0;
    loop {
        let curr_time = Instant::now();
        if curr_time - start_time > timeout_duration {
            break;
        }
        for c in buf {
            if c == b'\x03' {
                count += 1;
                if count == 3 {
                    return Ok(());
                }
            } else {
                count = 0;
                print!("{}", c);
            }
        }
        serial.read(&mut buf)?;
    }
    Err(SoapboxError::TimeoutError(timeout_duration.as_secs()))
}

fn load_payload<P: AsRef<Path>>(path: P) -> SoapboxResult<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

fn send_size(serial: &mut Serial, size: usize) -> SoapboxResult<()> {
    let size = size as u32;
    let size_le = u32::to_le_bytes(size);
    serial.write(&size_le)
}

fn send_payload(serial: &mut Serial, data: &[u8]) -> SoapboxResult<()> {
    const BYTES_PER_SLICE: usize = 512;
    for chunk in data.chunks(BYTES_PER_SLICE).progress() {
        serial.write(chunk)?;
    }
    Ok(())
}

fn terminal(serial: Serial) {
    let serial_recv = Arc::new(Mutex::new(serial));
    let serial_send = Arc::clone(&serial_recv);

    let (kill_send, kill_recv) = std::sync::mpsc::sync_channel(2);

    let receiver = thread::spawn(move || loop {
        if let Ok(_) = kill_recv.try_recv() {
            break;
        }
        let ch = {
            let mut serial = serial_recv.lock().unwrap();
            serial.getc().unwrap()
        };
        if ch == b'\n' {
            console::putc(b'\r').unwrap();
        }
        console::putc(ch).unwrap();
    });

    loop {
        let ch = console::getc().unwrap();
        if ch == b'\x03' {
            kill_send.send(()).unwrap();
            break;
        }
        let mut serial = serial_send.lock().unwrap();
        serial.putc(ch).unwrap();
    }

    receiver.join().unwrap();
}
