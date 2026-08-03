//! CRUD routes for rooms: `/api/rooms`.

use std::collections::HashMap;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};

use smart_home::Room;

use crate::AppState;
use crate::dto::{CreateRoom, RoomDto, room_to_dto};
use crate::error::ApiError;

/// Router for room operations (full paths, merged via `merge`).
pub fn rooms_routes() -> Router<AppState> {
    Router::new()
        .route("/api/rooms", get(list_rooms).post(create_room))
        .route("/api/rooms/{room_id}", get(get_room).delete(delete_room))
}

/// `GET /api/rooms` — list all rooms.
pub async fn list_rooms(State(state): State<AppState>) -> Json<Vec<RoomDto>> {
    let home = state.home.read().await;
    let mut rooms: Vec<RoomDto> = home
        .rooms()
        .map(|(id, room)| room_to_dto(id, room))
        .collect();
    rooms.sort_by(|a, b| a.id.cmp(&b.id));
    Json(rooms)
}

/// `POST /api/rooms` — add a room.
pub async fn create_room(
    State(state): State<AppState>,
    Json(body): Json<CreateRoom>,
) -> Result<(StatusCode, Json<RoomDto>), ApiError> {
    if body.id.trim().is_empty() {
        return Err(ApiError::BadRequest(
            "room id must not be empty".to_string(),
        ));
    }
    let room = Room::new(body.name.unwrap_or_else(|| body.id.clone()), HashMap::new());
    let dto = room_to_dto(&body.id, &room);
    let mut home = state.home.write().await;
    if home.room(&body.id).is_some() {
        return Err(ApiError::Conflict(format!(
            "room '{}' already exists",
            body.id
        )));
    }
    home.insert_room(body.id.clone(), room);
    Ok((StatusCode::CREATED, Json(dto)))
}

/// `GET /api/rooms/{room_id}` — room information.
pub async fn get_room(
    State(state): State<AppState>,
    Path(room_id): Path<String>,
) -> Result<Json<RoomDto>, ApiError> {
    let home = state.home.read().await;
    match home.room(&room_id) {
        Some(room) => Ok(Json(room_to_dto(&room_id, room))),
        None => Err(ApiError::NotFound(format!("room '{room_id}' not found"))),
    }
}

/// `DELETE /api/rooms/{room_id}` — delete a room.
pub async fn delete_room(
    State(state): State<AppState>,
    Path(room_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let mut home = state.home.write().await;
    if home.remove_room(&room_id).is_some() {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound(format!("room '{room_id}' not found")))
    }
}
