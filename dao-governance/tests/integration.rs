// =====================================================================================
// File: dao-governance/tests/integration.rs
// Description: Integration tests for the DAO governance microservice HTTP API.
// Author: arkSong (arksong2018@gmail.com)
// =====================================================================================

use actix_web::{test, App};
use dao_governance::{DaoStore, Member, Proposal, Vote};
use serde_json::json;
use actix_web::{web, get, post, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct AddMemberRequest {
    address: String,
}

#[derive(Debug, Deserialize)]
struct CreateProposalRequest {
    proposer: String,
    title: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct VoteRequest {
    proposal_id: u64,
    voter: String,
    support: bool,
}

#[derive(Debug, Deserialize)]
struct ExecuteProposalRequest {
    proposal_id: u64,
}

#[actix_web::test]
async fn test_member_proposal_vote_execute() {
    use dao_governance::{DaoStore};
    use actix_web::{App, web, get, post, HttpResponse, Responder};
    use chrono::Utc;

    #[post("/members")]
    async fn add_member(data: web::Data<DaoStore>, req: web::Json<AddMemberRequest>) -> impl Responder {
        match data.add_member(req.address.clone()) {
            Ok(member) => HttpResponse::Ok().json(member),
            Err(e) => HttpResponse::BadRequest().body(format!("Add member error: {}", e)),
        }
    }

    #[post("/proposals")]
    async fn create_proposal(data: web::Data<DaoStore>, req: web::Json<CreateProposalRequest>) -> impl Responder {
        match data.create_proposal(req.proposer.clone(), req.title.clone(), req.description.clone()) {
            Ok(proposal) => HttpResponse::Ok().json(proposal),
            Err(e) => HttpResponse::BadRequest().body(format!("Create proposal error: {}", e)),
        }
    }

    #[post("/vote")]
    async fn vote(data: web::Data<DaoStore>, req: web::Json<VoteRequest>) -> impl Responder {
        match data.vote(req.proposal_id, req.voter.clone(), req.support) {
            Ok(vote) => HttpResponse::Ok().json(vote),
            Err(e) => HttpResponse::BadRequest().body(format!("Vote error: {}", e)),
        }
    }

    #[post("/execute")]
    async fn execute_proposal(data: web::Data<DaoStore>, req: web::Json<ExecuteProposalRequest>) -> impl Responder {
        match data.execute_proposal(req.proposal_id) {
            Ok(proposal) => HttpResponse::Ok().json(proposal),
            Err(e) => HttpResponse::BadRequest().body(format!("Execute proposal error: {}", e)),
        }
    }

    #[get("/members")]
    async fn get_members(data: web::Data<DaoStore>) -> impl Responder {
        let members = data.get_members();
        HttpResponse::Ok().json(members)
    }

    #[get("/proposals")]
    async fn get_proposals(data: web::Data<DaoStore>) -> impl Responder {
        let proposals = data.get_proposals();
        HttpResponse::Ok().json(proposals)
    }

    let store = DaoStore::new();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(store.clone()))
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