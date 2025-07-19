// =====================================================================================
// File: audit-service/tests/integration.rs
// Description: Integration tests for the audit microservice HTTP API.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{test, App};
use audit_service::{AuditStore, AuditEvent};
use serde_json::json;
use actix_web::{web, get, post, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct LogEventRequest {
    event_type: String,
    actor: String,
    target: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct ComplianceCheckRequest {
    id: u64,
}

#[actix_web::test]
async fn test_log_list_compliance_report() {
    use audit_service::AuditStore;
    use actix_web::{App, web, get, post, HttpResponse, Responder};

    #[post("/audit/log")]
    async fn log_event(data: web::Data<AuditStore>, req: web::Json<LogEventRequest>) -> impl Responder {
        match data.log_event(req.event_type.clone(), req.actor.clone(), req.target.clone(), req.description.clone()) {
            Ok(event) => HttpResponse::Ok().json(event),
            Err(e) => HttpResponse::BadRequest().body(format!("Log event error: {}", e)),
        }
    }

    #[get("/audit/events")]
    async fn list_events(data: web::Data<AuditStore>) -> impl Responder {
        let events = data.list_events();
        HttpResponse::Ok().json(events)
    }

    #[post("/audit/compliance")]
    async fn compliance_check(data: web::Data<AuditStore>, req: web::Json<ComplianceCheckRequest>) -> impl Responder {
        let rule = |event: &AuditEvent| event.event_type != "forbidden";
        match data.compliance_check(req.id, rule) {
            Ok(event) => HttpResponse::Ok().json(event),
            Err(e) => HttpResponse::BadRequest().body(format!("Compliance check error: {}", e)),
        }
    }

    #[get("/audit/report")]
    async fn generate_report(data: web::Data<AuditStore>) -> impl Responder {
        let report = data.generate_report();
        HttpResponse::Ok().body(report)
    }

    let store = AuditStore::new();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(store.clone()))
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