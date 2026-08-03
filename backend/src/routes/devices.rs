//! Device routes within a room: `/api/rooms/{room_id}/devices`.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};

use smart_home::{Device, Power, Socket, Temperature, Thermometer};

use crate::AppState;
use crate::dto::{CreateDevice, DeviceDto, DeviceKind, device_to_dto};
use crate::error::ApiError;

/// Router for device operations (full paths, merged via `merge`).
pub fn devices_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/rooms/{room_id}/devices",
            get(list_devices).post(create_device),
        )
        .route(
            "/api/rooms/{room_id}/devices/{device_id}",
            get(get_device).delete(delete_device),
        )
        .route(
            "/api/rooms/{room_id}/devices/{device_id}/turn_on",
            post(turn_on),
        )
        .route(
            "/api/rooms/{room_id}/devices/{device_id}/turn_off",
            post(turn_off),
        )
}

/// `GET /api/rooms/{room_id}/devices` — list devices.
pub async fn list_devices(
    State(state): State<AppState>,
    Path(room_id): Path<String>,
) -> Result<Json<Vec<DeviceDto>>, ApiError> {
    let home = state.home.read().await;
    let room = home
        .room(&room_id)
        .ok_or_else(|| ApiError::NotFound(format!("room '{room_id}' not found")))?;
    let mut devices: Vec<DeviceDto> = room
        .devices()
        .map(|(id, device)| device_to_dto(id, device))
        .collect();
    devices.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(Json(devices))
}

/// `POST /api/rooms/{room_id}/devices` — add a device.
pub async fn create_device(
    State(state): State<AppState>,
    Path(room_id): Path<String>,
    Json(body): Json<CreateDevice>,
) -> Result<(StatusCode, Json<DeviceDto>), ApiError> {
    if body.id.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "device id must not be empty".to_string(),
        ));
    }
    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "device name must not be empty".to_string(),
        ));
    }
    let device = build_device(&body)?;
    let dto = device_to_dto(&body.id, &device);

    let mut home = state.home.write().await;
    let room = home
        .room_mut(&room_id)
        .ok_or_else(|| ApiError::NotFound(format!("room '{room_id}' not found")))?;
    if room.device(&body.id).is_some() {
        return Err(ApiError::Conflict(format!(
            "device '{}' already exists in room '{}'",
            body.id, room_id
        )));
    }
    room.insert_device(body.id.clone(), device);
    Ok((StatusCode::CREATED, Json(dto)))
}

/// `GET /api/rooms/{room_id}/devices/{device_id}` — device information.
pub async fn get_device(
    State(state): State<AppState>,
    Path((room_id, device_id)): Path<(String, String)>,
) -> Result<Json<DeviceDto>, ApiError> {
    let home = state.home.read().await;
    let device = home.device(&room_id, &device_id)?;
    Ok(Json(device_to_dto(&device_id, device)))
}

/// `DELETE /api/rooms/{room_id}/devices/{device_id}` — delete a device.
pub async fn delete_device(
    State(state): State<AppState>,
    Path((room_id, device_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    let mut home = state.home.write().await;
    let room = home
        .room_mut(&room_id)
        .ok_or_else(|| ApiError::NotFound(format!("room '{room_id}' not found")))?;
    if room.remove_device(&device_id).is_some() {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound(format!(
            "device '{device_id}' not found in room '{room_id}'"
        )))
    }
}

/// `POST /api/rooms/{room_id}/devices/{device_id}/turn_on` — turn a socket on.
pub async fn turn_on(
    State(state): State<AppState>,
    Path((room_id, device_id)): Path<(String, String)>,
) -> Result<Json<DeviceDto>, ApiError> {
    set_socket_state(&state, &room_id, &device_id, true).await
}

/// `POST /api/rooms/{room_id}/devices/{device_id}/turn_off` — turn a socket off.
pub async fn turn_off(
    State(state): State<AppState>,
    Path((room_id, device_id)): Path<(String, String)>,
) -> Result<Json<DeviceDto>, ApiError> {
    set_socket_state(&state, &room_id, &device_id, false).await
}

/// Changes the socket state and returns the updated DTO.
async fn set_socket_state(
    state: &AppState,
    room_id: &str,
    device_id: &str,
    on: bool,
) -> Result<Json<DeviceDto>, ApiError> {
    let mut home = state.home.write().await;
    let device = home.device_mut(room_id, device_id)?;
    match device {
        Device::Socket(socket) => {
            if on {
                socket.turn_on();
            } else {
                socket.turn_off();
            }
        }
        Device::Thermometer(_) => {
            return Err(ApiError::BadRequest(format!(
                "device '{device_id}' is a thermometer, not a socket"
            )));
        }
    }
    let device = home.device(room_id, device_id)?;
    Ok(Json(device_to_dto(device_id, device)))
}

/// Builds a domain device from the request body.
fn build_device(body: &CreateDevice) -> Result<Device, ApiError> {
    match body.kind {
        DeviceKind::Socket => {
            let is_on = body.is_on.unwrap_or(false);
            let watts = body.power_watts.unwrap_or(0.0);
            let power = Power::new(watts)
                .map_err(|e| ApiError::BadRequest(format!("invalid power: {e}")))?;
            Ok(Device::Socket(Socket::new(body.name.clone(), is_on, power)))
        }
        DeviceKind::Thermometer => {
            let celsius = body.temperature_celsius.unwrap_or(20.0);
            Ok(Device::Thermometer(Thermometer::new(
                body.name.clone(),
                Temperature::celsius(celsius),
            )))
        }
    }
}
