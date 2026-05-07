//! UDP temperature simulator: non-blocking send loop, config file driven.
//!
//! Config file format (UTF-8 text):
//! - line 1: destination `HOST:PORT` for UDP datagrams
//! - line 2: period in milliseconds between sends
//! - line 3 (optional): fixed temperature in °C; if omitted, each send uses a pseudo-random value
//!
//! Usage: `thermo_sim <CONFIG_PATH>`

use std::fs;
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

use smart_home::wire;

struct Config {
    dest: SocketAddr,
    period: Duration,
    fixed: Option<f32>,
}

fn parse_config(text: &str) -> Result<Config, String> {
    let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
    let dest_line = lines
        .next()
        .ok_or_else(|| "missing destination line".to_string())?;
    let dest: SocketAddr = dest_line
        .parse()
        .map_err(|e| format!("invalid destination: {e}"))?;
    let period_line = lines
        .next()
        .ok_or_else(|| "missing period line".to_string())?;
    let ms: u64 = period_line
        .parse()
        .map_err(|e| format!("invalid period ms: {e}"))?;
    let fixed = if let Some(t) = lines.next() {
        Some(
            t.parse::<f32>()
                .map_err(|e| format!("invalid temperature: {e}"))?,
        )
    } else {
        None
    };
    Ok(Config {
        dest,
        period: Duration::from_millis(ms),
        fixed,
    })
}

fn next_temp(fixed: Option<f32>, tick: u64) -> f32 {
    if let Some(t) = fixed {
        return t;
    }
    // deterministic "random" in a comfortable band for demos
    let x = ((tick.wrapping_mul(6364136223846793005) ^ 0x9E37_79B9) % 10_000) as f32 / 10_000.0;
    18.0 + x * 8.0
}

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| usage_and_exit());
    let text = fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("read config: {e}");
        std::process::exit(1);
    });
    let cfg = parse_config(&text).unwrap_or_else(|e| {
        eprintln!("config: {e}");
        std::process::exit(1);
    });

    let sock = UdpSocket::bind("0.0.0.0:0").unwrap_or_else(|e| {
        eprintln!("bind udp: {e}");
        std::process::exit(1);
    });
    sock.set_nonblocking(true).unwrap_or_else(|e| {
        eprintln!("set_nonblocking: {e}");
        std::process::exit(1);
    });

    eprintln!(
        "thermo_sim -> {} every {:?} (non-blocking)",
        cfg.dest, cfg.period
    );

    let mut next = Instant::now();
    let mut tick: u64 = 0;
    loop {
        let now = Instant::now();
        if now >= next {
            let t = next_temp(cfg.fixed, tick);
            tick = tick.wrapping_add(1);
            let payload = wire::encode_temperature_celsius(t);
            match sock.send_to(&payload, cfg.dest) {
                Ok(_) => {}
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => eprintln!("send: {e}"),
            }
            next = now.checked_add(cfg.period).unwrap_or(now + cfg.period);
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn usage_and_exit() -> ! {
    eprintln!("usage: thermo_sim <CONFIG_PATH>");
    std::process::exit(2);
}
