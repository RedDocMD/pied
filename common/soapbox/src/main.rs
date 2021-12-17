use std::{
    env, fs,
    path::Path,
    process,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use colored::*;
use error::SoapboxResult;
use indicatif::{HumanBytes, ProgressBar, ProgressStyle};
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
    ctrlc::set_handler(|| {
        println!("\n[{}] Bye 👋", SHORT_NAME);
        process::exit(1);
    })
    .expect("Error setting Ctrl+C handler");
    loop {
        if let Err(err) = soapbox(&args) {
            match err {
                SoapboxError::IOError(err) => fatal_error(err.to_string()),
                SoapboxError::SerialError(err) => fatal_error(err.to_string()),
                SoapboxError::TimeoutError(_) | SoapboxError::ProtocolError(_) => {
                    reconnect_message()
                }
            }
        } else {
            break;
        }
    }
}

fn reconnect_message() {
    println!(
        "\n[{}] ⚡ {}: {}",
        SHORT_NAME,
        "Connection or protocol Error".bright_red().bold(),
        "Remove power and USB serial. Reinsert serial first, then power".bright_red(),
    );
}

fn fatal_error(mess: String) -> ! {
    println!(
        "\n[{}] ⚡ {}: {}",
        SHORT_NAME,
        "Unexpected Error".bright_red().bold(),
        mess.bright_red()
    );
    process::exit(1);
}

fn soapbox(args: &[String]) -> SoapboxResult<()> {
    let serial_port_name = &args[1];
    let payload_path = &args[2];

    let mut serial = open_serial(serial_port_name);
    wait_for_payload_request(&mut serial)?;
    let data = load_payload(payload_path)?;
    send_size(&mut serial, data.len())?;
    send_payload(&mut serial, &data, payload_path)?;
    terminal(serial);
    Ok(())
}

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
    let mut count = 0;

    let start_time = Instant::now();
    let timeout_duration = Duration::from_secs(10);

    let mut len = serial.read_partial(&mut buf)?;
    loop {
        let curr_time = Instant::now();
        if curr_time - start_time > timeout_duration {
            break;
        }
        for c in &buf[..len] {
            if *c == b'\x03' {
                count += 1;
                if count == 3 {
                    return Ok(());
                }
            } else {
                count = 0;
                print!("{}", *c as char);
            }
        }
        len = serial.read_partial(&mut buf)?;
    }
    Err(SoapboxError::TimeoutError(timeout_duration.as_secs()))
}

fn load_payload<P: AsRef<Path>>(path: P) -> SoapboxResult<Vec<u8>> {
    Ok(fs::read(path)?)
}

fn send_size(serial: &mut Serial, size: usize) -> SoapboxResult<()> {
    let size = size as u32;
    let size_le = u32::to_le_bytes(size);
    serial.write(&size_le)?;
    let mut buf = [0_u8; 2];
    serial.read(&mut buf)?;
    if &buf != b"OK" {
        return Err(SoapboxError::ProtocolError(
            "Expected to receive \"OK\" after size was sent",
        ));
    }
    Ok(())
}

fn send_payload(serial: &mut Serial, data: &[u8], path: &str) -> SoapboxResult<()> {
    println!("[{}] ⏩ Sending {} ...", SHORT_NAME, path);
    const BYTES_PER_SLICE: usize = 512;
    let chunk_cnt = if data.len() % BYTES_PER_SLICE == 0 {
        data.len() / BYTES_PER_SLICE
    } else {
        data.len() / BYTES_PER_SLICE + 1
    };
    let bar = ProgressBar::new(chunk_cnt as u64).with_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "[{}] ⏩ Pushing {} {{bar:50}} {{percent}}% {{bytes_per_sec}} Time: {{eta_precise}}",
                SHORT_NAME,
                HumanBytes(data.len() as u64)
            ))
            .progress_chars("=🦀 "),
    );
    for chunk in data.chunks(BYTES_PER_SLICE) {
        serial.write(chunk)?;
        bar.inc(1);
    }
    bar.finish();
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
