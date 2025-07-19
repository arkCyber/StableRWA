// =====================================================================================
// File: audit-service/src/main.rs
// Description: Actix-web HTTP server for the audit microservice. Exposes REST API
//              endpoints for logging, querying, compliance checking, and reporting audit events.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{web, get, post, App, HttpServer, Responder, HttpResponse};
use chrono::Utc;
use log::{info, error};
use audit_service::{AuditStore, AuditEvent, AuditError};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct AppState {
    store: AuditStore,
}

/// Request body for logging an audit event
#[derive(Debug, Deserialize)]
struct LogEventRequest {
    event_type: String,
    actor: String,
    target: String,
    description: String,
}

/// Request body for compliance check
#[derive(Debug, Deserialize)]
struct ComplianceCheckRequest {
    id: u64,
}

/// POST /audit/log - Log a new audit event
#[post("/audit/log")]
async fn log_event(data: web::Data<AppState>, req: web::Json<LogEventRequest>) -> impl Responder {
    info!("[{}] POST /audit/log called", Utc::now());
    match data.store.log_event(req.event_type.clone(), req.actor.clone(), req.target.clone(), req.description.clone()) {
        Ok(event) => HttpResponse::Ok().json(event),
        Err(e) => {
            error!("[{}] Log event error: {}", Utc::now(), e);
            HttpResponse::BadRequest().body(format!("Log event error: {}", e))
        }
    }
}

/// GET /audit/events - List all audit events
#[get("/audit/events")]
async fn list_events(data: web::Data<AppState>) -> impl Responder {
    info!("[{}] GET /audit/events called", Utc::now());
    let events = data.store.list_events();
    HttpResponse::Ok().json(events)
}

/// POST /audit/compliance - Run compliance check on an event (simple demo rule)
#[post("/audit/compliance")]
async fn compliance_check(data: web::Data<AppState>, req: web::Json<ComplianceCheckRequest>) -> impl Responder {
    info!("[{}] POST /audit/compliance called", Utc::now());
    // Example compliance rule: event_type must not be "forbidden"
    let rule = |event: &AuditEvent| event.event_type != "forbidden";
    match data.store.compliance_check(req.id, rule) {
        Ok(event) => HttpResponse::Ok().json(event),
        Err(e) => {
            error!("[{}] Compliance check error: {}", Utc::now(), e);
            HttpResponse::BadRequest().body(format!("Compliance check error: {}", e))
        }
    }
}

/// GET /audit/report - Generate audit report
#[get("/audit/report")]
async fn generate_report(data: web::Data<AppState>) -> impl Responder {
    info!("[{}] GET /audit/report called", Utc::now());
    let report = data.store.generate_report();
    HttpResponse::Ok().body(report)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("[{}] Starting audit microservice on 0.0.0.0:8095", Utc::now());
    let state = AppState { store: AuditStore::new() };
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(log_event)
            .service(list_events)
            .service(compliance_check)
            .service(generate_report)
    })
    .bind(("0.0.0.0", 8095))?
    .run()
    .await
}

// ======================
// Tests for the server
// ======================
#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_log_list_compliance_report() {
        let state = AppState { store: AuditStore::new() };
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state.clone()))
                .service(log_event)
                .service(list_events)
                .service(compliance_check)
                .service(generate_report)
        ).await;
        // Log event
        let req = test::TestRequest::post().uri("/audit/log").set_json(&LogEventRequest {
            event_type: "login".to_string(),
            actor: "user1".to_string(),
            target: "system".to_string(),
            description: "User login".to_string(),
        }).to_request();
        let event: AuditEvent = test::call_and_read_body_json(&app, req).await;
        assert_eq!(event.event_type, "login");
        // List events
        let req = test::TestRequest::get().uri("/audit/events").to_request();
        let events: Vec<AuditEvent> = test::call_and_read_body_json(&app, req).await;
        assert!(!events.is_empty());
        // Compliance check
        let req = test::TestRequest::post().uri("/audit/compliance").set_json(&ComplianceCheckRequest {
            id: event.id,
        }).to_request();
        let checked: AuditEvent = test::call_and_read_body_json(&app, req).await;
        assert!(checked.compliance_checked);
        // Report
        let req = test::TestRequest::get().uri("/audit/report").to_request();
        let body = test::call_and_read_body(&app, req).await;
        let report = String::from_utf8(body.to_vec()).unwrap();
        assert!(report.contains("Total Events"));
    }
} 