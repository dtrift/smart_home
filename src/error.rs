use std::error::Error;
use std::fmt;

/// Error when resolving a room or device in [`crate::SmartHome`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmartHomeError {
    /// No room with the given key.
    RoomNotFound { room: String },
    /// Room exists, but no device with the given key.
    DeviceNotFound { room: String, device: String },
}

impl fmt::Display for SmartHomeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SmartHomeError::RoomNotFound { room } => {
                write!(f, "room not found: {room}")
            }
            SmartHomeError::DeviceNotFound { room, device } => {
                write!(f, "device not found: {device} (in room {room})")
            }
        }
    }
}

impl Error for SmartHomeError {}
