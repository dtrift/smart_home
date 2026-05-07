//! TCP smart-outlet simulator: non-blocking accept/read, length-prefixed frames, shared state, many clients.
//!
//! Usage: `socket_sim <BIND_ADDR> [--watts <W>] [--on|--off]`

use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use smart_home::wire;

#[derive(Debug, Clone)]
struct OutletState {
    is_on: bool,
    nominal_watts: f32,
}

impl OutletState {
    fn watts_now(&self) -> f32 {
        if self.is_on { self.nominal_watts } else { 0.0 }
    }
}

struct Client {
    stream: TcpStream,
    out: Vec<u8>,
    buf: Vec<u8>,
}

impl Client {
    fn new(stream: TcpStream) -> io::Result<Self> {
        stream.set_nonblocking(true)?;
        Ok(Self {
            stream,
            out: Vec::new(),
            buf: Vec::new(),
        })
    }

    fn push_response(&mut self, payload: &str) {
        let bytes = payload.as_bytes();
        let len = u32::try_from(bytes.len()).expect("socket_sim response frame is too large");
        self.out.extend_from_slice(&len.to_be_bytes());
        self.out.extend_from_slice(bytes);
    }

    fn flush_writes(&mut self) -> io::Result<()> {
        while !self.out.is_empty() {
            match self.stream.write(&self.out) {
                Ok(0) => {
                    return Err(io::Error::new(io::ErrorKind::WriteZero, "short write"));
                }
                Ok(n) => {
                    self.out.drain(..n);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn read_available(&mut self) -> io::Result<()> {
        let mut tmp = [0_u8; 512];
        loop {
            match self.stream.read(&mut tmp) {
                Ok(0) => {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "client closed",
                    ));
                }
                Ok(n) => self.buf.extend_from_slice(&tmp[..n]),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn drain_frames(&mut self, state: &Arc<Mutex<OutletState>>) -> io::Result<()> {
        while self.buf.len() >= 4 {
            let len =
                u32::from_be_bytes(self.buf[..4].try_into().expect("length prefix is 4 bytes"))
                    as usize;
            if len > 1024 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("frame is too large: {len} bytes"),
                ));
            }
            if self.buf.len() < 4 + len {
                break;
            }
            self.buf.drain(..4);
            let raw: Vec<u8> = self.buf.drain(..len).collect();
            let command = String::from_utf8(raw).map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, format!("bad UTF-8: {e}"))
            })?;
            match command.as_str() {
                "GET_STATUS" => {
                    let g = state
                        .lock()
                        .map_err(|_| io::Error::other("state mutex poisoned"))?;
                    let msg = wire::format_status_line(g.is_on, g.watts_now());
                    self.push_response(&msg);
                }
                "SET_ON" => {
                    if let Ok(mut g) = state.lock() {
                        g.is_on = true;
                    }
                    self.push_response("OK");
                }
                "SET_OFF" => {
                    if let Ok(mut g) = state.lock() {
                        g.is_on = false;
                    }
                    self.push_response("OK");
                }
                "" => {}
                other => {
                    self.push_response(&format!("ERR unknown command: {other}"));
                }
            }
        }
        Ok(())
    }
}

fn parse_args() -> Result<(SocketAddr, OutletState), String> {
    let mut args = std::env::args().skip(1);
    let bind: SocketAddr = args
        .next()
        .ok_or_else(|| "missing BIND_ADDR".to_string())?
        .parse()
        .map_err(|e| format!("invalid bind address: {e}"))?;

    let mut watts = 1500.0_f32;
    let mut is_on = false;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--watts" => {
                let v = args
                    .next()
                    .ok_or_else(|| "--watts needs a value".to_string())?;
                watts = v.parse().map_err(|e| format!("invalid watts: {e}"))?;
            }
            "--on" => is_on = true,
            "--off" => is_on = false,
            other => return Err(format!("unknown arg: {other}")),
        }
    }

    Ok((
        bind,
        OutletState {
            is_on,
            nominal_watts: watts,
        },
    ))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (bind, initial) =
        parse_args().map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let state = Arc::new(Mutex::new(initial));
    let listener = TcpListener::bind(bind)?;
    listener.set_nonblocking(true)?;
    eprintln!("socket_sim listening on {bind} (non-blocking)");

    let mut clients: Vec<Client> = Vec::new();

    loop {
        match listener.accept() {
            Ok((stream, _)) => match Client::new(stream) {
                Ok(c) => clients.push(c),
                Err(e) => eprintln!("client init: {e}"),
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(e.into()),
        }

        let mut i = 0;
        while i < clients.len() {
            let remove = {
                let c = &mut clients[i];
                match c.read_available() {
                    Err(_) => true,
                    Ok(()) => c.drain_frames(&state).is_err() || c.flush_writes().is_err(),
                }
            };
            if remove {
                clients.swap_remove(i);
            } else {
                i += 1;
            }
        }

        std::thread::sleep(Duration::from_millis(2));
    }
}
