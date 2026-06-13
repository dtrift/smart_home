use std::fmt;
use std::net::SocketAddr;
use std::sync::Mutex;

use super::device::DeviceInfo;
use crate::report::Report;
use crate::types::Power;
use crate::wire;

/// Smart socket: local state or synchronous TCP to a remote outlet.
#[derive(Debug)]
pub struct Socket {
    name: String,
    inner: SocketInner,
}

#[derive(Debug)]
enum SocketInner {
    Local {
        is_on: bool,
        power: Power,
    },
    Tcp {
        addr: SocketAddr,
        /// Rated power (watts) when the outlet is on; used if the remote does not echo watts.
        rated: Power,
        state: Mutex<TcpState>,
    },
}

#[derive(Debug, Clone)]
struct TcpState {
    is_on: bool,
    watts: f32,
    last_error: Option<String>,
}

impl Socket {
    /// Creates a new local socket (no network I/O).
    pub fn new(name: String, is_on: bool, power: Power) -> Self {
        Self {
            name,
            inner: SocketInner::Local { is_on, power },
        }
    }

    /// Connects to a remote smart outlet over TCP and reads its current state.
    ///
    /// This performs a synchronous `GET_STATUS` round-trip during construction.
    pub fn connect_tcp(name: String, addr: SocketAddr, rated: Power) -> std::io::Result<Self> {
        let (is_on, watts) = wire::socket_get_status(addr)?;
        Ok(Self {
            name,
            inner: SocketInner::Tcp {
                addr,
                rated,
                state: Mutex::new(TcpState {
                    is_on,
                    watts,
                    last_error: None,
                }),
            },
        })
    }

    /// Returns `true` if this socket uses TCP to talk to a remote device.
    pub fn is_remote(&self) -> bool {
        matches!(self.inner, SocketInner::Tcp { .. })
    }

    /// Last I/O or protocol error for TCP mode; `None` for local sockets or after a successful sync.
    pub fn last_error(&self) -> Option<String> {
        match &self.inner {
            SocketInner::Local { .. } => None,
            SocketInner::Tcp { state, .. } => state.lock().ok()?.last_error.clone(),
        }
    }

    fn sync_tcp_cache(addr: SocketAddr, _rated: Power, state: &Mutex<TcpState>) {
        let mut guard = match state.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        match wire::socket_get_status(addr) {
            Ok((on, w)) => {
                guard.is_on = on;
                guard.watts = w;
                guard.last_error = None;
            }
            Err(e) => {
                guard.last_error = Some(e.to_string());
            }
        }
    }

    fn tcp_apply<F>(addr: SocketAddr, state: &Mutex<TcpState>, op: F)
    where
        F: FnOnce() -> std::io::Result<()>,
    {
        let mut guard = match state.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        match op() {
            Ok(()) => match wire::socket_get_status(addr) {
                Ok((on, w)) => {
                    guard.is_on = on;
                    guard.watts = w;
                    guard.last_error = None;
                }
                Err(e) => {
                    guard.last_error = Some(e.to_string());
                }
            },
            Err(e) => {
                guard.last_error = Some(e.to_string());
            }
        }
    }

    pub fn turn_on(&mut self) {
        match &mut self.inner {
            SocketInner::Local { is_on, .. } => *is_on = true,
            SocketInner::Tcp { addr, state, .. } => {
                let addr = *addr;
                Self::tcp_apply(addr, state, || wire::socket_set_on(addr));
            }
        }
    }

    pub fn turn_off(&mut self) {
        match &mut self.inner {
            SocketInner::Local { is_on, .. } => *is_on = false,
            SocketInner::Tcp { addr, state, .. } => {
                let addr = *addr;
                Self::tcp_apply(addr, state, || wire::socket_set_off(addr));
            }
        }
    }

    pub fn is_on(&self) -> bool {
        match &self.inner {
            SocketInner::Local { is_on, .. } => *is_on,
            SocketInner::Tcp { addr, rated, state } => {
                let addr = *addr;
                let rated = *rated;
                Self::sync_tcp_cache(addr, rated, state);
                state.lock().map(|g| g.is_on).unwrap_or(false)
            }
        }
    }

    pub fn power(&self) -> Power {
        match &self.inner {
            SocketInner::Local { is_on, power } => {
                if *is_on {
                    *power
                } else {
                    Power::zero()
                }
            }
            SocketInner::Tcp { addr, rated, state } => {
                let addr = *addr;
                let rated = *rated;
                Self::sync_tcp_cache(addr, rated, state);
                let watts = state.lock().map(|g| g.watts).unwrap_or(0.0);
                Power::new(watts).unwrap_or_default()
            }
        }
    }
}

impl Clone for Socket {
    fn clone(&self) -> Self {
        match &self.inner {
            SocketInner::Local { is_on, power } => Self {
                name: self.name.clone(),
                inner: SocketInner::Local {
                    is_on: *is_on,
                    power: *power,
                },
            },
            SocketInner::Tcp { addr, rated, state } => Self {
                name: self.name.clone(),
                inner: SocketInner::Tcp {
                    addr: *addr,
                    rated: *rated,
                    state: Mutex::new(state.lock().map(|g| g.clone()).unwrap_or(TcpState {
                        is_on: false,
                        watts: 0.0,
                        last_error: Some("mutex poisoned".to_string()),
                    })),
                },
            },
        }
    }
}

impl Default for Socket {
    fn default() -> Self {
        Self::new("Socket".to_string(), false, Power::default())
    }
}

impl PartialEq for Socket {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name {
            return false;
        }
        match (&self.inner, &other.inner) {
            (
                SocketInner::Local {
                    is_on: a,
                    power: p1,
                },
                SocketInner::Local {
                    is_on: b,
                    power: p2,
                },
            ) => a == b && p1 == p2,
            (
                SocketInner::Tcp {
                    addr: aa,
                    rated: r1,
                    ..
                },
                SocketInner::Tcp {
                    addr: ab,
                    rated: r2,
                    ..
                },
            ) => aa == ab && r1 == r2,
            _ => false,
        }
    }
}

impl fmt::Display for Socket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Socket({})", self.name)
    }
}

impl DeviceInfo for Socket {
    fn name(&self) -> &str {
        &self.name
    }

    fn state(&self) -> String {
        let err = self.last_error();
        let base = format!(
            "Socket '{}': {} (power: {:.1} W)",
            self.name,
            if self.is_on() { "on" } else { "off" },
            self.power().watts()
        );
        match err {
            Some(e) => format!("{base} [error: {e}]"),
            None => base,
        }
    }
}

impl Report for Socket {
    fn report(&self) -> String {
        format!("{}\n", self.state())
    }
}

#[cfg(test)]
mod tests {
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[test]
    fn tcp_socket_turn_on_off() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let addr = listener.local_addr().unwrap();
        let flag = std::sync::Arc::new(std::sync::Mutex::new(false));
        let flag2 = flag.clone();
        let (tx, rx) = mpsc::channel::<()>();
        thread::spawn(move || {
            for _ in 0..32 {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let mut stream = stream;
                        stream
                            .set_read_timeout(Some(Duration::from_secs(2)))
                            .unwrap();
                        stream
                            .set_write_timeout(Some(Duration::from_secs(2)))
                            .unwrap();
                        while let Ok(command) = wire::read_frame(&mut stream) {
                            if command == "GET_STATUS" {
                                let on = *flag2.lock().unwrap();
                                let w = if on { 99.0 } else { 0.0 };
                                let msg = wire::format_status_line(on, w);
                                let _ = wire::write_frame(&mut stream, msg.as_bytes());
                            } else if command == "SET_ON" {
                                *flag2.lock().unwrap() = true;
                                let _ = wire::write_frame(&mut stream, b"OK");
                            } else if command == "SET_OFF" {
                                *flag2.lock().unwrap() = false;
                                let _ = wire::write_frame(&mut stream, b"OK");
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        if rx.try_recv().is_ok() {
                            break;
                        }
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => break,
                }
            }
        });

        thread::sleep(Duration::from_millis(50));

        let rated = Power::new(99.0).unwrap();
        let mut socket = Socket::connect_tcp("s".to_string(), addr, rated).expect("connect");
        assert!(!socket.is_on());
        socket.turn_on();
        assert!(socket.is_on());
        assert!((socket.power().watts() - 99.0).abs() < 0.01);
        socket.turn_off();
        assert!(!socket.is_on());
        let _ = tx.send(());
    }
}
