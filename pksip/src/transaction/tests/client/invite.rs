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

//////////////////////////////////
// Invite Client Transaction Tests
//////////////////////////////////

#[tokio::test]
async fn transitions_to_calling_when_request_sent() {
    let ctx = TestContextSendRequest::setup(SipMethod::Invite);

    let uac = ClientTransaction::send_request(&ctx.endpoint, ctx.request, Some(ctx.target))
        .await
        .expect("error sending request");

    assert_eq!(
        uac.state(),
        fsm::State::Calling,
        "should transition to calling after initiate the transaction"
    );
}

#[tokio::test(start_paused = true)]
async fn should_not_start_timer_a_when_transport_is_reliable() {
    let mut ctx = ReliableTransportTestContext::setup_async(SipMethod::Invite).await;
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
async fn transitions_from_calling_to_proceeding_when_receiving_1xx_response() {
    let mut ctx = ReceiveProvisionalTestContext::setup_async(SipMethod::Invite).await;

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
async fn transitions_from_calling_to_completed_when_receiving_3xx_response() {
    let mut ctx = ReceiveFinalTestContext::setup_async(SipMethod::Invite).await;

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
async fn transitions_from_calling_to_completed_when_receiving_4xx_response() {
    let mut ctx = ReceiveFinalTestContext::setup_async(SipMethod::Invite).await;

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
async fn transitions_from_calling_to_completed_when_receiving_5xx_response() {
    let mut ctx = ReceiveFinalTestContext::setup_async(SipMethod::Invite).await;

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
async fn transitions_from_calling_to_completed_when_receiving_6xx_response() {
    let mut ctx = ReceiveFinalTestContext::setup_async(SipMethod::Invite).await;

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
async fn transitions_from_calling_to_terminated_when_receiving_2xx_response() {
    let mut ctx = ReceiveFinalTestContext::setup_async(SipMethod::Invite).await;

    ctx.server.respond(STATUS_CODE_202_ACCEPTED).await;

    ctx.client
        .receive_final_response()
        .await
        .expect("Error receiving final response");

    assert_state_eq!(
        ctx.client_state,
        fsm::State::Terminated,
        "should transition to Terminated after receiving 2xx response"
    );
}

#[tokio::test(start_paused = true)]
async fn transitions_from_calling_to_terminated_when_timer_b_fires() {
    let mut ctx = ReceiveProvisionalTestContext::setup_async(SipMethod::Invite).await;

    let opt_err = ctx.client.receive_provisional_response().await.err();

    assert_matches!(
        opt_err,
        Some(Error::TransactionError(TransactionError::Timeout)),
        "Expected TransactionError::Timeout, got {opt_err:?}"
    );

    assert_eq!(
        ctx.client.state(),
        fsm::State::Terminated,
        "should transition to Terminated after timeout"
    );
}

#[tokio::test]
async fn should_send_ack_after_3xx_response_in_calling_state() {
    let ctx = RetransmissionTestContext::setup_async(SipMethod::Invite).await;

    ctx.server.respond(STATUS_CODE_301_MOVED_PERMANENTLY).await;

    ctx.client
        .receive_final_response()
        .await
        .expect("Error receiving final response");

    let req = ctx.transport.get_last_request().expect("A request");
    assert_eq!(
        req.method(),
        SipMethod::Ack,
        "MUST generate an ACK request after receiving 3xx response"
    );
}

#[tokio::test]
async fn should_send_ack_after_4xx_response_in_calling_state() {
    let ctx = RetransmissionTestContext::setup_async(SipMethod::Invite).await;

    ctx.server.respond(STATUS_CODE_404_NOT_FOUND).await;

    ctx.client
        .receive_final_response()
        .await
        .expect("Error receiving final response");

    let req = ctx.transport.get_last_request().expect("A request");
    assert_eq!(
        req.method(),
        SipMethod::Ack,
        "MUST generate an ACK request after receiving 4xx response"
    );
}

#[tokio::test]
async fn should_send_ack_after_5xx_response_in_calling_state() {
    let ctx = RetransmissionTestContext::setup_async(SipMethod::Invite).await;

    ctx.server.respond(STATUS_CODE_504_SERVER_TIMEOUT).await;

    ctx.client
        .receive_final_response()
        .await
        .expect("Error receiving final response");

    let req = ctx.transport.get_last_request().expect("A request");
    assert_eq!(
        req.method(),
        SipMethod::Ack,
        "MUST generate an ACK request after receiving 5xx response"
    );
}

#[tokio::test]
async fn should_send_ack_after_6xx_response_in_calling_state() {
    let ctx = RetransmissionTestContext::setup_async(SipMethod::Invite).await;

    ctx.server.respond(STATUS_CODE_603_DECLINE).await;

    ctx.client
        .receive_final_response()
        .await
        .expect("Error receiving final response");

    let req = ctx.transport.get_last_request().expect("A request");
    assert_eq!(
        req.method(),
        SipMethod::Ack,
        "MUST generate an ACK request after receiving 6xx response"
    );
}

#[tokio::test]
async fn transitions_from_proceeding_to_completed_when_receiving_3xx_response() {
    let mut ctx = ReceiveFinalTestContext::setup_async(SipMethod::Invite).await;

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
    let mut ctx = ReceiveFinalTestContext::setup_async(SipMethod::Invite).await;

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
    let mut ctx = ReceiveFinalTestContext::setup_async(SipMethod::Invite).await;

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
    let mut ctx = ReceiveFinalTestContext::setup_async(SipMethod::Invite).await;

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
async fn transitions_from_proceeding_to_terminated_when_receiving_2xx_response() {
    let mut ctx = ReceiveFinalTestContext::setup_async(SipMethod::Invite).await;

    ctx.server.respond(STATUS_CODE_202_ACCEPTED).await;

    ctx.client
        .receive_final_response()
        .await
        .expect("Error receiving final response");

    assert_state_eq!(
        ctx.client_state,
        fsm::State::Terminated,
        "should transition to Terminated after receiving 2xx response"
    );
}

#[tokio::test(start_paused = true)]
async fn should_not_retransmit_request_in_proceeding_state() {
    let mut ctx = RetransmissionTestContext::setup_async(SipMethod::Invite).await;
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
        expected_requests + expected_retrans
    );
}

#[tokio::test]
async fn should_send_ack_after_3xx_response_in_proceeding_state() {
    let mut ctx = RetransmissionTestContext::setup_async(SipMethod::Invite).await;

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

    ctx.server.respond(STATUS_CODE_301_MOVED_PERMANENTLY).await;

    ctx.client
        .receive_final_response()
        .await
        .expect("Error receiving final response");

    let req = ctx.transport.get_last_request().expect("A request");
    assert_eq!(
        req.method(),
        SipMethod::Ack,
        "MUST generate an ACK request after receiving 3xx response"
    );
}

#[tokio::test]
async fn should_send_ack_after_4xx_response_in_proceeding_state() {
    let mut ctx = RetransmissionTestContext::setup_async(SipMethod::Invite).await;

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

    ctx.server.respond(STATUS_CODE_404_NOT_FOUND).await;

    ctx.client
        .receive_final_response()
        .await
        .expect("Error receiving final response");

    let req = ctx.transport.get_last_request().expect("A request");
    assert_eq!(
        req.method(),
        SipMethod::Ack,
        "MUST generate an ACK request after receiving 4xx response"
    );
}

#[tokio::test]
async fn should_send_ack_after_5xx_response_in_proceeding_state() {
    let mut ctx = RetransmissionTestContext::setup_async(SipMethod::Invite).await;

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

    ctx.server.respond(STATUS_CODE_504_SERVER_TIMEOUT).await;

    ctx.client
        .receive_final_response()
        .await
        .expect("Error receiving final response");

    let req = ctx.transport.get_last_request().expect("A request");
    assert_eq!(
        req.method(),
        SipMethod::Ack,
        "MUST generate an ACK request after receiving 5xx response"
    );
}

#[tokio::test]
async fn should_send_ack_after_6xx_response_in_proceeding_state() {
    let mut ctx = RetransmissionTestContext::setup_async(SipMethod::Invite).await;

    ctx.server.respond(STATUS_CODE_100_TRYING).await;

    ctx.client
        .receive_provisional_response()
        .await
        .expect("Error receiving provisional response")
        .expect("Expected provisional response, but received None");

    assert_eq!(
        ctx.client.state(),
        fsm::State::Proceeding,
        "should transition to Proceeding after receiving 1xx"
    );

    ctx.server.respond(STATUS_CODE_603_DECLINE).await;

    ctx.client
        .receive_final_response()
        .await
        .expect("Error receiving final response");

    let req = ctx.transport.get_last_request().expect("A request");
    assert_eq!(
        req.method(),
        SipMethod::Ack,
        "MUST generate an ACK request after receiving 6xx response"
    );
}

#[tokio::test]
async fn should_pass_provisional_responses_to_tu_in_proceeding_state() {
    let mut ctx = ReceiveProvisionalTestContext::setup_async(SipMethod::Invite).await;

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
async fn transitions_from_completed_to_terminated_when_timer_d_fires() {
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
