//! Report route: `/api/report`.

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use smart_home::Report;

use crate::AppState;
use crate::dto::ReportDto;

/// Router for the home report (full path, merged via `merge`).
pub fn report_routes() -> Router<AppState> {
    Router::new().route("/api/report", get(report))
}

/// `GET /api/report` — textual report of the whole home.
pub async fn report(State(state): State<AppState>) -> Json<ReportDto> {
    let home = state.home.read().await;
    Json(ReportDto {
        report: home.report(),
    })
}
