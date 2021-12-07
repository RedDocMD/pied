use std::{
    env,
    error::Error,
    fmt::{self, Display, Formatter},
    fs::File,
    io::Read,
    path::Path,
    process, thread,
    time::{Duration, Instant},
};

use colored::*;
use serial::Serial;

mod console;
mod serial;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: {} <serial-port> <input-file>", args[0]);
        process::exit(1);
    }
    let mut serial = open_serial(&args[1]);
    let data = match load_payload(&args[1]) {
        Ok(data) => data,
        Err(err) => unexpected_error(&err),
    };
    if let Err(err) = wait_for_payload_request(&mut serial) {
        unexpected_error(&err);
    }
}

fn unexpected_error(err: &anyhow::Error) -> ! {
    println!(
        "\n[{}] ⚡ {}",
        SHORT_NAME,
        format!("Unexpected Error: {}", err).bright_red()
    );
    process::exit(1);
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

fn wait_for_payload_request(serial: &mut Serial) -> anyhow::Result<()> {
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
    Err(TimeoutError {
        duration: timeout_duration,
    })?
}

#[derive(Debug)]
struct TimeoutError {
    duration: Duration,
}

impl Error for TimeoutError {}

impl Display for TimeoutError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "timed out on {}s", self.duration.as_secs())
    }
}

fn load_payload<P: AsRef<Path>>(path: P) -> anyhow::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}
