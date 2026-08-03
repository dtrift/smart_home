//! REST API functional tests: spin up the server and assert HTTP responses.

use backend::{AppState, api_router};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use smart_home::SmartHome;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

fn empty_state() -> AppState {
    AppState {
        home: Arc::new(RwLock::new(SmartHome::new(
            "Test Home".to_string(),
            HashMap::new(),
        ))),
    }
}

/// Spawns the API router on a random port and returns the base URL.
async fn spawn_server() -> String {
    let state = empty_state();
    let app = api_router().with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

#[derive(Debug, Serialize, Deserialize)]
struct RoomDto {
    id: String,
    name: String,
    device_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct DeviceDto {
    id: String,
    kind: String,
    name: String,
    #[serde(default)]
    is_on: Option<bool>,
    #[serde(default)]
    power_watts: Option<f32>,
    #[serde(default)]
    temperature_celsius: Option<f32>,
}

#[derive(Debug, Serialize)]
struct CreateRoom {
    id: String,
    name: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateDevice {
    id: String,
    kind: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_on: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    power_watts: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature_celsius: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct ReportDto {
    report: String,
}

#[tokio::test]
async fn full_api_workflow() {
    let base = spawn_server().await;
    let client = reqwest::Client::new();

    // Rooms list is empty.
    let rooms: Vec<RoomDto> = client
        .get(format!("{base}/api/rooms"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(rooms.is_empty());

    // Create a room.
    let created: RoomDto = client
        .post(format!("{base}/api/rooms"))
        .json(&CreateRoom {
            id: "Kitchen".to_string(),
            name: Some("Kitchen".to_string()),
        })
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(created.id, "Kitchen");
    assert_eq!(created.name, "Kitchen");
    assert_eq!(created.device_count, 0);

    // Fetch the room.
    let got: RoomDto = client
        .get(format!("{base}/api/rooms/Kitchen"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(got.id, "Kitchen");

    // 404 for a missing room.
    let status = client
        .get(format!("{base}/api/rooms/Nope"))
        .send()
        .await
        .unwrap()
        .status();
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Conflict on duplicate creation.
    let status = client
        .post(format!("{base}/api/rooms"))
        .json(&CreateRoom {
            id: "Kitchen".to_string(),
            name: None,
        })
        .send()
        .await
        .unwrap()
        .status();
    assert_eq!(status, StatusCode::CONFLICT);

    // Devices list is empty.
    let devs: Vec<DeviceDto> = client
        .get(format!("{base}/api/rooms/Kitchen/devices"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(devs.is_empty());

    // Add a socket.
    let socket: DeviceDto = client
        .post(format!("{base}/api/rooms/Kitchen/devices"))
        .json(&CreateDevice {
            id: "kettle".to_string(),
            kind: "socket".to_string(),
            name: "Kettle".to_string(),
            is_on: Some(false),
            power_watts: Some(1500.0),
            temperature_celsius: None,
        })
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(socket.id, "kettle");
    assert_eq!(socket.kind, "socket");
    assert_eq!(socket.is_on, Some(false));
    assert_eq!(socket.power_watts, Some(0.0)); // off → 0 W

    // Turn the socket on.
    let on: DeviceDto = client
        .post(format!("{base}/api/rooms/Kitchen/devices/kettle/turn_on"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(on.is_on, Some(true));
    assert_eq!(on.power_watts, Some(1500.0));

    // Turn the socket off.
    let off: DeviceDto = client
        .post(format!("{base}/api/rooms/Kitchen/devices/kettle/turn_off"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(off.is_on, Some(false));
    assert_eq!(off.power_watts, Some(0.0));

    // Add a thermometer.
    let thermo: DeviceDto = client
        .post(format!("{base}/api/rooms/Kitchen/devices"))
        .json(&CreateDevice {
            id: "sensor".to_string(),
            kind: "thermometer".to_string(),
            name: "Sensor".to_string(),
            is_on: None,
            power_watts: None,
            temperature_celsius: Some(21.5),
        })
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(thermo.kind, "thermometer");
    assert_eq!(thermo.temperature_celsius, Some(21.5));

    // turn_on on a thermometer → 400.
    let status = client
        .post(format!("{base}/api/rooms/Kitchen/devices/sensor/turn_on"))
        .send()
        .await
        .unwrap()
        .status();
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Invalid power → 400.
    let status = client
        .post(format!("{base}/api/rooms/Kitchen/devices"))
        .json(&CreateDevice {
            id: "bad".to_string(),
            kind: "socket".to_string(),
            name: "Bad".to_string(),
            is_on: Some(true),
            power_watts: Some(-5.0),
            temperature_celsius: None,
        })
        .send()
        .await
        .unwrap()
        .status();
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Device information.
    let got: DeviceDto = client
        .get(format!("{base}/api/rooms/Kitchen/devices/kettle"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(got.id, "kettle");

    // 404 for a device in a missing room.
    let status = client
        .get(format!("{base}/api/rooms/Nope/devices/kettle"))
        .send()
        .await
        .unwrap()
        .status();
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Devices list now contains 2 items.
    let devs: Vec<DeviceDto> = client
        .get(format!("{base}/api/rooms/Kitchen/devices"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(devs.len(), 2);

    // Report contains the room name.
    let rep: ReportDto = client
        .get(format!("{base}/api/report"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(rep.report.contains("Kitchen"));

    // Delete the device.
    let status = client
        .delete(format!("{base}/api/rooms/Kitchen/devices/kettle"))
        .send()
        .await
        .unwrap()
        .status();
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Repeated deletion → 404.
    let status = client
        .delete(format!("{base}/api/rooms/Kitchen/devices/kettle"))
        .send()
        .await
        .unwrap()
        .status();
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Delete the room.
    let status = client
        .delete(format!("{base}/api/rooms/Kitchen"))
        .send()
        .await
        .unwrap()
        .status();
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Rooms list is empty again.
    let rooms: Vec<RoomDto> = client
        .get(format!("{base}/api/rooms"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(rooms.is_empty());
}
