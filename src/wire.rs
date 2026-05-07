//! Line-oriented TCP commands for smart sockets and UDP payload for thermometers.

use crate::types::Temperature;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

/// Default timeout for synchronous TCP operations from library code.
pub const TCP_TIMEOUT: Duration = Duration::from_secs(2);

pub fn tcp_connect(addr: impl ToSocketAddrs) -> io::Result<TcpStream> {
    let stream = TcpStream::connect_timeout(
        &addr
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "empty address list"))?,
        TCP_TIMEOUT,
    )?;
    stream.set_read_timeout(Some(TCP_TIMEOUT))?;
    stream.set_write_timeout(Some(TCP_TIMEOUT))?;
    Ok(stream)
}

pub fn read_line(stream: &mut TcpStream) -> io::Result<String> {
    let mut buf = Vec::new();
    let mut byte = [0_u8; 1];
    loop {
        let n = stream.read(&mut byte)?;
        if n == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "connection closed before newline",
            ));
        }
        if byte[0] == b'\n' {
            break;
        }
        buf.push(byte[0]);
    }
    String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn write_line(stream: &mut TcpStream, line: &[u8]) -> io::Result<()> {
    stream.write_all(line)?;
    if !line.ends_with(b"\n") {
        stream.write_all(b"\n")?;
    }
    stream.flush()
}

pub fn socket_get_status(addr: SocketAddr) -> io::Result<(bool, f32)> {
    let mut stream = tcp_connect(addr)?;
    write_line(&mut stream, b"GET_STATUS")?;
    let line = read_line(&mut stream)?;
    parse_status_line(&line)
}

pub fn socket_set_on(addr: SocketAddr) -> io::Result<()> {
    let mut stream = tcp_connect(addr)?;
    write_line(&mut stream, b"SET_ON")?;
    expect_ok(&mut stream)
}

pub fn socket_set_off(addr: SocketAddr) -> io::Result<()> {
    let mut stream = tcp_connect(addr)?;
    write_line(&mut stream, b"SET_OFF")?;
    expect_ok(&mut stream)
}

fn expect_ok(stream: &mut TcpStream) -> io::Result<()> {
    let line = read_line(stream)?;
    if line.trim() == "OK" {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("expected OK, got {line:?}"),
        ))
    }
}

fn parse_status_line(line: &str) -> io::Result<(bool, f32)> {
    let line = line.trim();
    let rest = line
        .strip_prefix("STATUS,")
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "bad STATUS prefix"))?;
    let mut parts = rest.splitn(2, ',');
    let flag = parts
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "STATUS missing on/off field"))?;
    let watts = parts
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "STATUS missing watts"))?
        .parse::<f32>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let on = match flag {
        "1" | "on" | "ON" | "true" | "TRUE" => true,
        "0" | "off" | "OFF" | "false" | "FALSE" => false,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("bad on/off flag: {flag}"),
            ));
        }
    };
    Ok((on, watts))
}

pub fn format_status_line(is_on: bool, watts: f32) -> String {
    let flag = if is_on { "1" } else { "0" };
    format!("STATUS,{flag},{watts}\n")
}

/// UDP payload: 4-byte little-endian IEEE754 `f32` (degrees Celsius).
pub fn encode_temperature_celsius(c: f32) -> [u8; 4] {
    c.to_le_bytes()
}

pub fn decode_temperature_celsius(buf: &[u8]) -> Option<Temperature> {
    if buf.len() < 4 {
        return None;
    }
    let mut arr = [0_u8; 4];
    arr.copy_from_slice(&buf[..4]);
    let v = f32::from_le_bytes(arr);
    Some(Temperature::celsius(v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_status_roundtrip() {
        let s = "STATUS,1,123.5";
        let (on, w) = parse_status_line(s).unwrap();
        assert!(on);
        assert!((w - 123.5).abs() < 0.01);
        let line = format_status_line(true, 123.5);
        let (on2, w2) = parse_status_line(line.trim_end()).unwrap();
        assert!(on2);
        assert!((w2 - 123.5).abs() < 0.01);
    }

    #[test]
    fn temperature_udp_roundtrip() {
        let b = encode_temperature_celsius(-3.5);
        let t = decode_temperature_celsius(&b).unwrap();
        assert!((t.as_celsius() + 3.5).abs() < 0.0001);
    }
}
