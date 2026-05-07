use std::fmt;
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use super::device::DeviceInfo;
use crate::report::Report;
use crate::types::Temperature;
use crate::wire;

/// Background UDP receiver shared by cloned handles; joins its thread when the last `Arc` is dropped.
struct UdpRecv {
    last: Arc<Mutex<Option<Temperature>>>,
    received: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    handle: Mutex<Option<JoinHandle<()>>>,
}

impl UdpRecv {
    fn spawn(bind: SocketAddr) -> io::Result<Arc<Self>> {
        let socket = UdpSocket::bind(bind)?;
        socket.set_read_timeout(Some(Duration::from_millis(200)))?;
        let last = Arc::new(Mutex::new(None));
        let received = Arc::new(AtomicBool::new(false));
        let stop = Arc::new(AtomicBool::new(false));

        let last_t = Arc::clone(&last);
        let recv_t = Arc::clone(&received);
        let stop_t = Arc::clone(&stop);
        let handle = std::thread::spawn(move || {
            let mut buf = [0_u8; 256];
            loop {
                if stop_t.load(Ordering::SeqCst) {
                    break;
                }
                match socket.recv_from(&mut buf) {
                    Ok((n, _)) => {
                        if let Some(t) = wire::decode_temperature_celsius(&buf[..n]) {
                            if let Ok(mut g) = last_t.lock() {
                                *g = Some(t);
                            }
                            recv_t.store(true, Ordering::SeqCst);
                        }
                    }
                    Err(ref e)
                        if e.kind() == io::ErrorKind::WouldBlock
                            || e.kind() == io::ErrorKind::TimedOut =>
                    {
                        continue;
                    }
                    Err(_) => continue,
                }
            }
        });

        Ok(Arc::new(Self {
            last,
            received,
            stop,
            handle: Mutex::new(Some(handle)),
        }))
    }
}

impl Drop for UdpRecv {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Ok(mut g) = self.handle.lock()
            && let Some(h) = g.take()
        {
            let _ = h.join();
        }
    }
}

enum ThermometerInner {
    Local {
        temperature: Temperature,
    },
    Udp {
        initial: Temperature,
        shared: Arc<UdpRecv>,
    },
}

impl fmt::Debug for ThermometerInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThermometerInner::Local { temperature } => f
                .debug_struct("Local")
                .field("temperature", temperature)
                .finish(),
            ThermometerInner::Udp { initial, shared } => f
                .debug_struct("Udp")
                .field("initial", initial)
                .field("shared", &format_args!("{:p}", Arc::as_ptr(shared)))
                .finish(),
        }
    }
}

/// Thermometer: local value or UDP-fed last reading in a background thread.
pub struct Thermometer {
    name: String,
    inner: ThermometerInner,
}

impl Thermometer {
    /// Local thermometer (no network thread).
    pub fn new(name: String, temperature: Temperature) -> Self {
        Self {
            name,
            inner: ThermometerInner::Local { temperature },
        }
    }

    /// Listens for UDP temperature packets on `bind` in a background thread.
    ///
    /// Until the first packet arrives, [`Self::temperature`] returns `initial`.
    pub fn bind_udp(name: String, bind: SocketAddr, initial: Temperature) -> io::Result<Self> {
        let shared = UdpRecv::spawn(bind)?;
        Ok(Self {
            name,
            inner: ThermometerInner::Udp { initial, shared },
        })
    }

    pub fn is_udp(&self) -> bool {
        matches!(self.inner, ThermometerInner::Udp { .. })
    }

    /// `true` if at least one UDP datagram was decoded successfully.
    pub fn has_udp_reading(&self) -> bool {
        match &self.inner {
            ThermometerInner::Local { .. } => true,
            ThermometerInner::Udp { shared, .. } => shared.received.load(Ordering::SeqCst),
        }
    }

    pub fn temperature(&self) -> Temperature {
        match &self.inner {
            ThermometerInner::Local { temperature } => *temperature,
            ThermometerInner::Udp { initial, shared } => {
                shared.last.lock().ok().and_then(|g| *g).unwrap_or(*initial)
            }
        }
    }
}

impl Clone for Thermometer {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            inner: match &self.inner {
                ThermometerInner::Local { temperature } => ThermometerInner::Local {
                    temperature: *temperature,
                },
                ThermometerInner::Udp { initial, shared } => ThermometerInner::Udp {
                    initial: *initial,
                    shared: Arc::clone(shared),
                },
            },
        }
    }
}

impl Default for Thermometer {
    fn default() -> Self {
        Self::new("Thermometer".to_string(), Temperature::default())
    }
}

impl PartialEq for Thermometer {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name {
            return false;
        }
        match (&self.inner, &other.inner) {
            (
                ThermometerInner::Local { temperature: a },
                ThermometerInner::Local { temperature: b },
            ) => a == b,
            (
                ThermometerInner::Udp { shared: s1, .. },
                ThermometerInner::Udp { shared: s2, .. },
            ) => Arc::ptr_eq(s1, s2),
            _ => false,
        }
    }
}

impl fmt::Debug for Thermometer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Thermometer")
            .field("name", &self.name)
            .field("inner", &self.inner)
            .finish()
    }
}

impl DeviceInfo for Thermometer {
    fn name(&self) -> &str {
        &self.name
    }

    fn state(&self) -> String {
        format!(
            "Thermometer '{}': {:.1}°C",
            self.name,
            self.temperature().as_celsius()
        )
    }
}

impl Report for Thermometer {
    fn report(&self) -> String {
        format!("{}\n", self.state())
    }
}

#[cfg(test)]
mod tests {
    use std::net::UdpSocket;
    use std::thread;
    use std::time::Duration;

    use super::*;

    #[test]
    fn udp_thermometer_receives() {
        let addr: SocketAddr = {
            let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
            sock.local_addr().unwrap()
        };

        let t = Thermometer::bind_udp("t".to_string(), addr, Temperature::celsius(1.0)).unwrap();
        assert!(!t.has_udp_reading());
        assert!((t.temperature().as_celsius() - 1.0).abs() < 0.01);

        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        let payload = wire::encode_temperature_celsius(19.25);
        sender.send_to(&payload, addr).unwrap();

        for _ in 0..50 {
            if t.has_udp_reading() {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(t.has_udp_reading());
        assert!((t.temperature().as_celsius() - 19.25).abs() < 0.01);
    }
}
