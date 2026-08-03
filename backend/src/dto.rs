//! DTO structs for the REST API and mapping from the library domain types.

use serde::{Deserialize, Serialize};

use smart_home::{Device, DeviceInfo, Room};

/// Room information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomDto {
    pub id: String,
    pub name: String,
    pub device_count: usize,
}

/// Device information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceDto {
    pub id: String,
    pub kind: DeviceKind,
    pub name: String,
    pub state: String,
    /// Socket only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_on: Option<bool>,
    /// Socket only (watts).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power_watts: Option<f32>,
    /// Thermometer only (°C).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature_celsius: Option<f32>,
}

/// Device kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceKind {
    Socket,
    Thermometer,
}

/// Request body for `POST /api/rooms`.
#[derive(Debug, Deserialize)]
pub struct CreateRoom {
    pub id: String,
    pub name: Option<String>,
}

/// Request body for `POST /api/rooms/{room_id}/devices`.
#[derive(Debug, Deserialize)]
pub struct CreateDevice {
    pub id: String,
    pub kind: DeviceKind,
    pub name: String,
    /// Socket only.
    #[serde(default)]
    pub is_on: Option<bool>,
    /// Socket only (watts, >= 0).
    #[serde(default)]
    pub power_watts: Option<f32>,
    /// Thermometer only (°C).
    #[serde(default)]
    pub temperature_celsius: Option<f32>,
}

/// Response body for `GET /api/report`.
#[derive(Debug, Serialize)]
pub struct ReportDto {
    pub report: String,
}

/// Builds a `RoomDto` from a key and a library room.
pub fn room_to_dto(id: &str, room: &Room) -> RoomDto {
    RoomDto {
        id: id.to_string(),
        name: room.name().to_string(),
        device_count: room.device_count(),
    }
}

/// Builds a `DeviceDto` from a key and a library device.
pub fn device_to_dto(id: &str, device: &Device) -> DeviceDto {
    let name = device.name().to_string();
    let state = device.state();
    match device {
        Device::Socket(s) => DeviceDto {
            id: id.to_string(),
            kind: DeviceKind::Socket,
            name,
            state,
            is_on: Some(s.is_on()),
            power_watts: Some(s.power().watts()),
            temperature_celsius: None,
        },
        Device::Thermometer(t) => DeviceDto {
            id: id.to_string(),
            kind: DeviceKind::Thermometer,
            name,
            state,
            is_on: None,
            power_watts: None,
            temperature_celsius: Some(t.temperature().as_celsius()),
        },
    }
}
