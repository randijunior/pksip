use crate::{
    SipMethod, assert_state_eq,
    error::{Error, TransactionError},
    test_utils::TestContext,
    transaction::{
        ClientTransaction,
        fsm::{self},
        tests::{
            STATUS_CODE_100_TRYING, STATUS_CODE_180_RINGING, STATUS_CODE_202_ACCEPTED,
            STATUS_CODE_301_MOVED_PERMANENTLY, STATUS_CODE_404_NOT_FOUND,
            STATUS_CODE_504_SERVER_TIMEOUT, STATUS_CODE_603_DECLINE,
        },
    },
};

use super::{
    ReceiveFinalTestContext, ReceiveProvisionalTestContext, ReliableTransportTestContext,
    RetransmissionTestContext, TestContextSendRequest,
};

#[tokio::test]
async fn transitions_to_trying_when_request_sent() {
    let ctx = TestContextSendRequest::setup(SipMethod::Bye);

    let uac = ClientTransaction::send_request(&ctx.endpoint, ctx.request, Some(ctx.target))
        .await
        .expect("failure sending request");

    assert_eq!(
        uac.state(),
        fsm::State::Trying,
        "should transition to trying ctx.client_state after initiating a new transaction and sending the request."
    );
}

#[tokio::test(start_paused = true)]
async fn should_not_start_timer_e_when_transport_is_reliable() {
    let mut ctx = ReliableTransportTestContext::setup_async(SipMethod::Options).await;
    let expected_requests = 1;
    let expected_retrans = 0;

    let opt_err = ctx.client.receive_provisional_response().await.err();

    assert_matches!(
        opt_err,
        Some(Error::TransactionError(TransactionError::Timeout)),
        "Expected TransactionError::Timeout, got {opt_err:?}"
    );

    assert_eq!(
        ctx.transport.sent_count(),
        expected_requests + expected_retrans,
        "sent count should match {expected_requests} requests and {expected_retrans} retransmissions"
    );
}

#[tokio::test]
async fn transitions_from_trying_to_proceeding_when_receiving_1xx_response() {
    let mut ctx = ReceiveProvisionalTestContext::setup_async(SipMethod::Register).await;

    ctx.server.respond(STATUS_CODE_100_TRYING).await;

    ctx.client
        .receive_provisional_response()
        .await
        .expect("Error receiving provisional response")
        .expect("Expected provisional response, but received None");

    assert_eq!(
        ctx.client.state(),
        fsm::State::Proceeding,
        "should transition to Proceeding after receiving 1xx response"
    );
}

#[tokio::test]
async fn transitions_from_trying_to_completed_when_receiving_2xx_response() {
    let mut ctx = ReceiveFinalTestContext::setup_async(SipMethod::Options).await;

    ctx.server.respond(STATUS_CODE_301_MOVED_PERMANENTLY).await;

    ctx.client
        .receive_final_response()
        .await
        .expect("Error receiving final response");

    assert_state_eq!(
        ctx.client_state,
        fsm::State::Completed,
        "should transition to Completed after receiving 6xx response"
    );
}

#[tokio::test]
async fn transitions_from_trying_to_completed_when_receiving_3xx_response() {
    let mut ctx = ReceiveFinalTestContext::setup_async(SipMethod::Options).await;

    ctx.server.respond(STATUS_CODE_301_MOVED_PERMANENTLY).await;

    ctx.client
        .receive_final_response()
        .await
        .expect("Error receiving final response");

    assert_state_eq!(
        ctx.client_state,
        fsm::State::Completed,
        "should transition to Completed after receiving 3xx response"
    );
}

#[tokio::test]
async fn transitions_from_trying_to_completed_when_receiving_4xx_response() {
    let mut ctx = ReceiveFinalTestContext::setup_async(SipMethod::Options).await;

    ctx.server.respond(STATUS_CODE_404_NOT_FOUND).await;

    ctx.client
        .receive_final_response()
        .await
        .expect("Error receiving final response");

    assert_state_eq!(
        ctx.client_state,
        fsm::State::Completed,
        "should transition to Completed after receiving 4xx response"
    );
}

#[tokio::test]
async fn transitions_from_trying_to_completed_when_receiving_5xx_response() {
    let mut ctx = ReceiveFinalTestContext::setup_async(SipMethod::Options).await;

    ctx.server.respond(STATUS_CODE_504_SERVER_TIMEOUT).await;

    ctx.client
        .receive_final_response()
        .await
        .expect("Error receiving final response");

    assert_state_eq!(
        ctx.client_state,
        fsm::State::Completed,
        "should transition to Completed after receiving 5xx response"
    );
}

#[tokio::test]
async fn transitions_from_trying_to_completed_when_receiving_6xx_response() {
    let mut ctx = ReceiveFinalTestContext::setup_async(SipMethod::Register).await;

    ctx.server.respond(STATUS_CODE_603_DECLINE).await;

    ctx.client
        .receive_final_response()
        .await
        .expect("Error receiving final response");

    assert_state_eq!(
        ctx.client_state,
        fsm::State::Completed,
        "should transition to Completed after receiving 6xx response"
    );
}
#[tokio::test]
async fn transitions_from_proceeding_to_completed_when_receiving_3xx_response() {
    let mut ctx = ReceiveFinalTestContext::setup_async(SipMethod::Options).await;

    ctx.server.respond(STATUS_CODE_301_MOVED_PERMANENTLY).await;

    ctx.client
        .receive_final_response()
        .await
        .expect("Error receiving final response");

    assert_state_eq!(
        ctx.client_state,
        fsm::State::Completed,
        "should transition to Completed after receiving 3xx response"
    );
}

#[tokio::test]
async fn transitions_from_proceeding_to_completed_when_receiving_4xx_response() {
    let mut ctx = ReceiveFinalTestContext::setup_async(SipMethod::Options).await;

    ctx.server.respond(STATUS_CODE_404_NOT_FOUND).await;

    ctx.client
        .receive_final_response()
        .await
        .expect("Error receiving final response");

    assert_state_eq!(
        ctx.client_state,
        fsm::State::Completed,
        "should transition to Completed after receiving 4xx response"
    );
}

#[tokio::test]
async fn transitions_from_proceeding_to_completed_when_receiving_5xx_response() {
    let mut ctx = ReceiveFinalTestContext::setup_async(SipMethod::Options).await;

    ctx.server.respond(STATUS_CODE_504_SERVER_TIMEOUT).await;

    ctx.client
        .receive_final_response()
        .await
        .expect("Error receiving final response");

    assert_state_eq!(
        ctx.client_state,
        fsm::State::Completed,
        "should transition to Completed after receiving 5xx response"
    );
}

#[tokio::test]
async fn transitions_from_proceeding_to_completed_when_receiving_6xx_response() {
    let mut ctx = ReceiveFinalTestContext::setup_async(SipMethod::Options).await;

    ctx.server.respond(STATUS_CODE_603_DECLINE).await;

    ctx.client
        .receive_final_response()
        .await
        .expect("Error receiving final response");

    assert_state_eq!(
        ctx.client_state,
        fsm::State::Completed,
        "should transition to Completed after receiving 6xx response"
    );
}

#[tokio::test]
async fn transitions_from_proceeding_to_completed_when_receiving_2xx_response() {
    let mut ctx = ReceiveFinalTestContext::setup_async(SipMethod::Options).await;

    ctx.server.respond(STATUS_CODE_202_ACCEPTED).await;

    ctx.client
        .receive_final_response()
        .await
        .expect("Error receiving final response");

    assert_state_eq!(
        ctx.client_state,
        fsm::State::Completed,
        "should transition to Completed after receiving 2xx response"
    );
}

#[tokio::test(start_paused = true)]
async fn transitions_from_trying_to_terminated_when_timer_f_fires() {
    let mut ctx = ReceiveProvisionalTestContext::setup_async(SipMethod::Register).await;

    let opt_err = ctx.client.receive_provisional_response().await.err();

    assert_matches!(
        opt_err,
        Some(Error::TransactionError(TransactionError::Timeout)),
        "Expected TransactionError::Timeout, got {opt_err:?}"
    );

    assert_eq!(
        ctx.client.state(),
        fsm::State::Terminated,
        "should transition to Terminated after timer f fires"
    );
}

#[tokio::test(start_paused = true)]
async fn should_not_retransmit_request_in_proceeding_state() {
    let mut ctx = RetransmissionTestContext::setup_async(SipMethod::Options).await;
    let expected_requests = 1;
    let expected_retrans = 0;

    ctx.server.respond(STATUS_CODE_100_TRYING).await;

    ctx.client
        .receive_provisional_response()
        .await
        .expect("Error receiving provisional response")
        .expect("Expected provisional response, but received None");

    assert_eq!(
        ctx.client.state(),
        fsm::State::Proceeding,
        "should transition to Proceeding after receiving 1xx response"
    );

    ctx.timer.wait_for_retransmissions(5).await;

    assert_eq!(
        ctx.transport.sent_count(),
        expected_requests + expected_retrans,
        "sent count should match {expected_requests} requests and {expected_retrans} retransmissions"
    );
}

#[tokio::test]
async fn should_pass_provisional_responses_to_tu_in_proceeding_state() {
    let mut ctx = ReceiveProvisionalTestContext::setup_async(SipMethod::Options).await;

    ctx.server.respond(STATUS_CODE_100_TRYING).await;
    ctx.server.respond(STATUS_CODE_180_RINGING).await;

    let response = ctx
        .client
        .receive_provisional_response()
        .await
        .expect("Error receiving provisional response")
        .expect("Expected provisional response, but received None");

    assert_eq!(
        response.status_code(),
        STATUS_CODE_100_TRYING,
        "should match 100 status code"
    );

    let response = ctx
        .client
        .receive_provisional_response()
        .await
        .expect("Error receiving provisional response")
        .expect("Expected provisional response, but received None");

    assert_eq!(
        response.status_code(),
        STATUS_CODE_180_RINGING,
        "should match 180 status code"
    );
}

#[tokio::test(start_paused = true)]
async fn transitions_from_completed_to_terminated_when_timer_k_fires() {
    let mut ctx = ReceiveFinalTestContext::setup_async(SipMethod::Invite).await;

    ctx.server.respond(STATUS_CODE_301_MOVED_PERMANENTLY).await;

    ctx.client
        .receive_final_response()
        .await
        .expect("Error receiving final response");

    tokio::time::sleep(64 * crate::transaction::T1).await;
    tokio::task::yield_now().await;

    assert_state_eq!(
        ctx.client_state,
        fsm::State::Terminated,
        "should transition to Terminated after timer d fires"
    );
}
