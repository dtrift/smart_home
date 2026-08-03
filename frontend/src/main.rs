//! Dioxus CSR frontend for the smart home: rooms, devices, socket control, report.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

// ---------- DTO (mirror of backend) ----------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct RoomDto {
    id: String,
    name: String,
    device_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
struct ReportDto {
    report: String,
}

// ---------- HTTP helpers ----------

async fn get_json<T: serde::de::DeserializeOwned>(url: &str) -> Result<T, String> {
    gloo_net::http::Request::get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<T>()
        .await
        .map_err(|e| e.to_string())
}

async fn post_json<T: serde::de::DeserializeOwned, B: serde::Serialize>(
    url: &str,
    body: &B,
) -> Result<T, String> {
    gloo_net::http::Request::post(url)
        .json(body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<T>()
        .await
        .map_err(|e| e.to_string())
}

async fn delete(url: &str) -> Result<(), String> {
    gloo_net::http::Request::delete(url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ---------- Navigation state ----------

#[derive(Clone, Debug, PartialEq)]
enum View {
    Rooms,
    Room(String),
    Device(String, String),
    Report,
}

// ---------- Entry point ----------

fn main() {
    dioxus::logger::initialize_default();
    launch(App);
}

fn App() -> Element {
    use_context_provider(|| Signal::new(View::Rooms));
    let mut current_view: Signal<View> = use_context();

    let body: Element = match current_view().clone() {
        View::Rooms => rsx! { RoomsView {} },
        View::Room(id) => rsx! { RoomView { room_id: id } },
        View::Device(room, dev) => rsx! { DeviceView { room_id: room, device_id: dev } },
        View::Report => rsx! { ReportView {} },
    };

    rsx! {
        header {
            h1 { "🏠 Smart Home" }
            nav {
                button {
                    onclick: move |_| current_view.set(View::Rooms),
                    "Rooms"
                }
                button {
                    onclick: move |_| current_view.set(View::Report),
                    "Report"
                }
            }
            hr {}
        }
        main { {body} }
    }
}

// ---------- View: rooms list ----------

#[component]
fn RoomsView() -> Element {
    let mut rooms: Resource<Result<Vec<RoomDto>, String>> =
        use_resource(|| async { get_json::<Vec<RoomDto>>("/api/rooms").await });
    let mut new_id = use_signal(String::new);
    let mut new_name = use_signal(String::new);

    let rooms_state: Option<Result<Vec<RoomDto>, String>> = rooms.read().clone();
    let list_view: Element = match &rooms_state {
        Some(Ok(list)) if list.is_empty() => rsx! { p { "No rooms." } },
        Some(Ok(list)) => {
            let items: Vec<Element> = list
                .iter()
                .map(|r: &RoomDto| {
                    let r = r.clone();
                    rsx! { RoomCard { room: r, on_refresh: move |_| rooms.restart() } }
                })
                .collect();
            rsx! { {items.into_iter()} }
        }
        Some(Err(e)) => rsx! { p { "Error: {e}" } },
        None => rsx! { p { "Loading…" } },
    };

    rsx! {
        h2 { "Rooms" }
        {list_view}

        div { class: "card",
            h3 { "Add room" }
            input { placeholder: "id", value: "{new_id}", oninput: move |e| new_id.set(e.value()) }
            input { placeholder: "name (optional)", value: "{new_name}", oninput: move |e| new_name.set(e.value()) }
            button {
                onclick: move |_| {
                    let id = new_id.read().clone();
                    if id.is_empty() {
                        return;
                    }
                    let name = new_name.read().clone();
                    let body = CreateRoom {
                        id,
                        name: if name.is_empty() { None } else { Some(name) },
                    };
                    spawn(async move {
                        let _ = post_json::<RoomDto, _>("/api/rooms", &body).await;
                        new_id.set(String::new());
                        new_name.set(String::new());
                        rooms.restart();
                    });
                },
                "Add"
            }
        }
    }
}

#[component]
fn RoomCard(room: RoomDto, on_refresh: EventHandler<()>) -> Element {
    let mut current_view: Signal<View> = use_context();
    let id_for_del = room.id.clone();
    let id_for_nav = room.id.clone();

    rsx! {
        div { class: "card",
            a {
                onclick: move |_| current_view.set(View::Room(id_for_nav.clone())),
                strong { "{room.name}" }
            }
            span { " ({room.device_count} devices)" }
            button {
                onclick: move |_| {
                    let id = id_for_del.clone();
                    spawn(async move {
                        let _ = delete(&format!("/api/rooms/{id}")).await;
                        on_refresh.call(());
                    });
                },
                "Delete"
            }
        }
    }
}

// ---------- View: room ----------

#[component]
fn RoomView(room_id: String) -> Element {
    let room_id_for_resource = room_id.clone();
    let mut devices: Resource<Result<Vec<DeviceDto>, String>> = use_resource(move || {
        let room_id = room_id_for_resource.clone();
        async move {
            let url = format!("/api/rooms/{}/devices", urlencoding(&room_id));
            get_json::<Vec<DeviceDto>>(&url).await
        }
    });
    let mut current_view: Signal<View> = use_context();
    let mut new_id = use_signal(String::new);
    let mut new_name = use_signal(String::new);
    let mut new_kind = use_signal(|| "socket".to_string());
    let mut new_value = use_signal(String::new);
    let room_id_clone = room_id.clone();

    let devices_state: Option<Result<Vec<DeviceDto>, String>> = devices.read().clone();
    let list_view: Element = match &devices_state {
        Some(Ok(list)) if list.is_empty() => rsx! { p { "No devices." } },
        Some(Ok(list)) => {
            let items: Vec<Element> = list
                .iter()
                .map(|d: &DeviceDto| {
                    let d = d.clone();
                    rsx! {
                        DeviceRow {
                            device: d,
                            room_id: room_id.clone(),
                            on_refresh: move |_| devices.restart(),
                        }
                    }
                })
                .collect();
            rsx! { {items.into_iter()} }
        }
        Some(Err(e)) => rsx! { p { "Error: {e}" } },
        None => rsx! { p { "Loading…" } },
    };

    rsx! {
        button { onclick: move |_| current_view.set(View::Rooms), "← Back to rooms" }
        h2 { "{room_id}" }
        {list_view}

        div { class: "card",
            h3 { "Add device" }
            input { placeholder: "id", value: "{new_id}", oninput: move |e| new_id.set(e.value()) }
            input { placeholder: "name", value: "{new_name}", oninput: move |e| new_name.set(e.value()) }
            select {
                value: "{new_kind}",
                onchange: move |e| new_kind.set(e.value()),
                option { value: "socket", "socket" }
                option { value: "thermometer", "thermometer" }
            }
            input {
                placeholder: "W (socket) / °C (thermometer)",
                value: "{new_value}",
                oninput: move |e| new_value.set(e.value()),
            }
            button {
                onclick: move |_| {
                    let id = new_id.read().clone();
                    if id.is_empty() {
                        return;
                    }
                    let kind = new_kind.read().clone();
                    let name = new_name.read().clone();
                    let val = new_value.read().parse::<f32>().ok();
                    let body = CreateDevice {
                        id,
                        kind: kind.clone(),
                        name,
                        is_on: if kind == "socket" { Some(false) } else { None },
                        power_watts: if kind == "socket" { val } else { None },
                        temperature_celsius: if kind == "thermometer" { val } else { None },
                    };
                    let room_id = room_id_clone.clone();
                    spawn(async move {
                        let url = format!("/api/rooms/{}/devices", urlencoding(&room_id));
                        let _ = post_json::<DeviceDto, _>(&url, &body).await;
                        new_id.set(String::new());
                        new_name.set(String::new());
                        new_value.set(String::new());
                        devices.restart();
                    });
                },
                "Add"
            }
        }
    }
}

#[component]
fn DeviceRow(device: DeviceDto, room_id: String, on_refresh: EventHandler<()>) -> Element {
    let mut current_view: Signal<View> = use_context();
    let dev_id_nav = device.id.clone();
    let dev_id_del = device.id.clone();
    let room_id_del = room_id.clone();

    let status = match device.is_on {
        Some(true) => " — on".to_string(),
        Some(false) => " — off".to_string(),
        None => String::new(),
    };

    rsx! {
        div { class: "card",
            a {
                onclick: move |_| {
                    current_view.set(View::Device(room_id.clone(), dev_id_nav.clone()))
                },
                strong { "{device.name}" }
            }
            span { " ({device.kind})" }
            span { "{status}" }
            button {
                onclick: move |_| {
                    let id = dev_id_del.clone();
                    let room = room_id_del.clone();
                    spawn(async move {
                        let _ = delete(&format!("/api/rooms/{}/devices/{id}", urlencoding(&room))).await;
                        on_refresh.call(());
                    });
                },
                "Delete"
            }
        }
    }
}

// ---------- View: device ----------

#[component]
fn DeviceView(room_id: String, device_id: String) -> Element {
    let (room_r, dev_r) = (room_id.clone(), device_id.clone());
    let mut device: Resource<Result<DeviceDto, String>> = use_resource(move || {
        let room = room_r.clone();
        let dev = dev_r.clone();
        async move {
            let url = format!(
                "/api/rooms/{}/devices/{}",
                urlencoding(&room),
                urlencoding(&dev)
            );
            get_json::<DeviceDto>(&url).await
        }
    });
    let mut current_view: Signal<View> = use_context();
    let room_back = room_id.clone();

    let device_state: Option<Result<DeviceDto, String>> = device.read().clone();
    let content: Element = match &device_state {
        Some(Ok(d)) => {
            let room_on = room_id.clone();
            let dev_on = device_id.clone();
            let room_off = room_id.clone();
            let dev_off = device_id.clone();
            let kind = d.kind.clone();
            let name = d.name.clone();
            let status = match d.is_on {
                Some(true) => "On".to_string(),
                Some(false) => "Off".to_string(),
                None => String::new(),
            };
            rsx! {
                div { class: "card",
                    h2 { "{name}" }
                    p { "Type: {kind}" }
                    if kind == "socket" {
                        p { "{status}" }
                        button {
                            onclick: move |_| {
                                let room = room_on.clone();
                                let dev = dev_on.clone();
                                spawn(async move {
                                    let url = format!(
                                        "/api/rooms/{}/devices/{}/turn_on",
                                        urlencoding(&room),
                                        urlencoding(&dev)
                                    );
                                    let _ = post_json::<DeviceDto, _>(&url, &serde_json::Value::Null).await;
                                    device.restart();
                                });
                            },
                            "Turn on"
                        }
                        button {
                            onclick: move |_| {
                                let room = room_off.clone();
                                let dev = dev_off.clone();
                                spawn(async move {
                                    let url = format!(
                                        "/api/rooms/{}/devices/{}/turn_off",
                                        urlencoding(&room),
                                        urlencoding(&dev)
                                    );
                                    let _ = post_json::<DeviceDto, _>(&url, &serde_json::Value::Null).await;
                                    device.restart();
                                });
                            },
                            "Turn off"
                        }
                    }
                }
            }
        }
        Some(Err(e)) => rsx! { p { "Error: {e}" } },
        None => rsx! { p { "Loading…" } },
    };

    rsx! {
        button {
            onclick: move |_| current_view.set(View::Room(room_back.clone())),
            "← Back to room"
        }
        {content}
    }
}

// ---------- View: report ----------

#[component]
fn ReportView() -> Element {
    let mut report: Resource<Result<ReportDto, String>> =
        use_resource(|| async { get_json::<ReportDto>("/api/report").await });

    let report_state: Option<Result<ReportDto, String>> = report.read().clone();
    let content: Element = match &report_state {
        Some(Ok(r)) => rsx! { pre { "{r.report}" } },
        Some(Err(e)) => rsx! { p { "Error: {e}" } },
        None => rsx! { p { "Loading…" } },
    };

    rsx! {
        h2 { "Home report" }
        button { onclick: move |_| report.restart(), "Refresh" }
        {content}
    }
}

// ---------- URL-encoding utility ----------

fn urlencoding(s: &str) -> String {
    // Minimal percent-encoding for spaces in path segments.
    s.replace(' ', "%20")
}
