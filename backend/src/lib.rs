//! Smart home REST backend (axum): CRUD for rooms/devices, socket control, report.

use std::collections::HashMap;
use std::sync::Arc;

use axum::Router;
use smart_home::{Power, Room, SmartHome, Socket, Temperature, Thermometer};
use tokio::sync::RwLock;
use tower_http::services::{ServeDir, ServeFile};

pub mod dto;
pub mod error;
pub mod routes;

/// Shared application state: the smart home behind a read/write lock.
#[derive(Clone)]
pub struct AppState {
    pub home: Arc<RwLock<SmartHome>>,
}

impl AppState {
    /// An empty home with the given name.
    pub fn empty(name: &str) -> Self {
        Self {
            home: Arc::new(RwLock::new(SmartHome::new(
                name.to_string(),
                HashMap::new(),
            ))),
        }
    }
}

/// State preloaded with demo data (as in `src/main.rs`).
pub fn demo_state() -> AppState {
    let kitchen = Room::new(
        "Kitchen".to_string(),
        devices([
            (
                "kitchen_thermometer".to_string(),
                Thermometer::new(
                    "Kitchen thermometer".to_string(),
                    Temperature::celsius(22.5),
                )
                .into(),
            ),
            (
                "kettle".to_string(),
                Socket::new("Kettle".to_string(), true, Power::new(1500.0).unwrap()).into(),
            ),
            (
                "fridge".to_string(),
                Socket::new("Fridge".to_string(), true, Power::new(200.0).unwrap()).into(),
            ),
        ]),
    );
    let living_room = Room::new(
        "Living Room".to_string(),
        devices([
            (
                "living_thermometer".to_string(),
                Thermometer::new(
                    "Living room thermometer".to_string(),
                    Temperature::celsius(24.0),
                )
                .into(),
            ),
            (
                "tv".to_string(),
                Socket::new("TV".to_string(), true, Power::new(120.0).unwrap()).into(),
            ),
            (
                "floor_lamp".to_string(),
                Socket::new("Floor lamp".to_string(), false, Power::new(60.0).unwrap()).into(),
            ),
        ]),
    );
    let bedroom = Room::new(
        "Bedroom".to_string(),
        devices([
            (
                "bedroom_thermometer".to_string(),
                Thermometer::new(
                    "Bedroom thermometer".to_string(),
                    Temperature::celsius(21.0),
                )
                .into(),
            ),
            (
                "humidifier".to_string(),
                Socket::new("Humidifier".to_string(), true, Power::new(30.0).unwrap()).into(),
            ),
        ]),
    );

    let mut rooms = HashMap::new();
    rooms.insert("Kitchen".to_string(), kitchen);
    rooms.insert("Living Room".to_string(), living_room);
    rooms.insert("Bedroom".to_string(), bedroom);

    AppState {
        home: Arc::new(RwLock::new(SmartHome::new(
            "My Smart Home".to_string(),
            rooms,
        ))),
    }
}

/// Collects a `HashMap` of devices from (key, device) pairs.
fn devices<I>(items: I) -> HashMap<String, smart_home::Device>
where
    I: IntoIterator<Item = (String, smart_home::Device)>,
{
    items.into_iter().collect()
}

/// API-only router (no static assets) — handy for tests.
pub fn api_router() -> Router<AppState> {
    Router::new()
        .merge(routes::rooms_routes())
        .merge(routes::devices_routes())
        .merge(routes::report_routes())
}

/// Full application router: API + frontend static assets.
pub fn app(state: AppState) -> Router {
    // ServeDir serves the built Dioxus frontend from frontend/dist; fallback to index.html.
    // If dist is missing (frontend not built), static paths return 404.
    let serve_dir =
        ServeDir::new("frontend/dist").fallback(ServeFile::new("frontend/dist/index.html"));
    api_router().with_state(state).fallback_service(serve_dir)
}
