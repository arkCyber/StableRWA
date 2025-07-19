// =====================================================================================
// File: dao-governance/src/main.rs
// Description: Actix-web HTTP server for the DAO governance microservice. Exposes REST API
//              endpoints for member management, proposal, voting, and execution.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{web, get, post, App, HttpServer, Responder, HttpResponse};
use chrono::Utc;
use log::{info, error};
use dao_governance::{DaoStore, Member, Proposal, Vote, DaoError};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct AppState {
    store: DaoStore,
}

/// Request body for adding a member
#[derive(Debug, Deserialize)]
struct AddMemberRequest {
    address: String,
}

/// Request body for creating a proposal
#[derive(Debug, Deserialize)]
struct CreateProposalRequest {
    proposer: String,
    title: String,
    description: String,
}

/// Request body for voting
#[derive(Debug, Deserialize)]
struct VoteRequest {
    proposal_id: u64,
    voter: String,
    support: bool,
}

/// Request body for executing a proposal
#[derive(Debug, Deserialize)]
struct ExecuteProposalRequest {
    proposal_id: u64,
}

/// POST /members - Add a new member
#[post("/members")]
async fn add_member(data: web::Data<AppState>, req: web::Json<AddMemberRequest>) -> impl Responder {
    info!("[{}] POST /members called", Utc::now());
    match data.store.add_member(req.address.clone()) {
        Ok(member) => HttpResponse::Ok().json(member),
        Err(e) => {
            error!("[{}] Add member error: {}", Utc::now(), e);
            HttpResponse::BadRequest().body(format!("Add member error: {}", e))
        }
    }
}

/// GET /members - List all members
#[get("/members")]
async fn get_members(data: web::Data<AppState>) -> impl Responder {
    info!("[{}] GET /members called", Utc::now());
    let members = data.store.get_members();
    HttpResponse::Ok().json(members)
}

/// POST /proposals - Create a new proposal
#[post("/proposals")]
async fn create_proposal(data: web::Data<AppState>, req: web::Json<CreateProposalRequest>) -> impl Responder {
    info!("[{}] POST /proposals called", Utc::now());
    match data.store.create_proposal(req.proposer.clone(), req.title.clone(), req.description.clone()) {
        Ok(proposal) => HttpResponse::Ok().json(proposal),
        Err(e) => {
            error!("[{}] Create proposal error: {}", Utc::now(), e);
            HttpResponse::BadRequest().body(format!("Create proposal error: {}", e))
        }
    }
}

/// GET /proposals - List all proposals
#[get("/proposals")]
async fn get_proposals(data: web::Data<AppState>) -> impl Responder {
    info!("[{}] GET /proposals called", Utc::now());
    let proposals = data.store.get_proposals();
    HttpResponse::Ok().json(proposals)
}

/// POST /vote - Cast a vote
#[post("/vote")]
async fn vote(data: web::Data<AppState>, req: web::Json<VoteRequest>) -> impl Responder {
    info!("[{}] POST /vote called", Utc::now());
    match data.store.vote(req.proposal_id, req.voter.clone(), req.support) {
        Ok(vote) => HttpResponse::Ok().json(vote),
        Err(e) => {
            error!("[{}] Vote error: {}", Utc::now(), e);
            HttpResponse::BadRequest().body(format!("Vote error: {}", e))
        }
    }
}

/// POST /execute - Execute a proposal
#[post("/execute")]
async fn execute_proposal(data: web::Data<AppState>, req: web::Json<ExecuteProposalRequest>) -> impl Responder {
    info!("[{}] POST /execute called", Utc::now());
    match data.store.execute_proposal(req.proposal_id) {
        Ok(proposal) => HttpResponse::Ok().json(proposal),
        Err(e) => {
            error!("[{}] Execute proposal error: {}", Utc::now(), e);
            HttpResponse::BadRequest().body(format!("Execute proposal error: {}", e))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    info!("[{}] Starting DAO governance microservice on 0.0.0.0:8093", Utc::now());
    let state = AppState { store: DaoStore::new() };
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .service(add_member)
            .service(get_members)
            .service(create_proposal)
            .service(get_proposals)
            .service(vote)
            .service(execute_proposal)
    })
    .bind(("0.0.0.0", 8093))?
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
    async fn test_member_proposal_vote_execute() {
        let state = AppState { store: DaoStore::new() };
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state.clone()))
                .service(add_member)
                .service(get_members)
                .service(create_proposal)
                .service(get_proposals)
                .service(vote)
                .service(execute_proposal)
        ).await;
        // Add member
        let req = test::TestRequest::post().uri("/members").set_json(&AddMemberRequest { address: "0xabc".to_string() }).to_request();
        let member: Member = test::call_and_read_body_json(&app, req).await;
        assert_eq!(member.address, "0xabc");
        // Create proposal
        let req = test::TestRequest::post().uri("/proposals").set_json(&CreateProposalRequest {
            proposer: "0xabc".to_string(),
            title: "Test Proposal".to_string(),
            description: "Desc".to_string(),
        }).to_request();
        let proposal: Proposal = test::call_and_read_body_json(&app, req).await;
        // Vote
        let req = test::TestRequest::post().uri("/vote").set_json(&VoteRequest {
            proposal_id: proposal.id,
            voter: "0xabc".to_string(),
            support: true,
        }).to_request();
        let vote: Vote = test::call_and_read_body_json(&app, req).await;
        assert_eq!(vote.proposal_id, proposal.id);
        // Execute
        let req = test::TestRequest::post().uri("/execute").set_json(&ExecuteProposalRequest {
            proposal_id: proposal.id,
        }).to_request();
        let executed: Proposal = test::call_and_read_body_json(&app, req).await;
        assert!(executed.executed);
    }
} 