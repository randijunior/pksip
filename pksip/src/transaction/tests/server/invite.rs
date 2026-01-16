use crate::{
    SipMethod, assert_state_eq,
    test_utils::transaction::TestContext,
    transaction::{
        fsm,
        tests::{STATUS_CODE_202_ACCEPTED, STATUS_CODE_301_MOVED_PERMANENTLY},
    },
};

use super::{
    ReliableTransportTestContext, RetransmissionTestContext, UnreliableTransportTestContext,
};

// ===== transaction state tests =====

#[tokio::test]
async fn transitions_to_confirmed_state_after_receive_ack() {
    let mut ctx = RetransmissionTestContext::setup(SipMethod::Invite);

    ctx.server
        .respond_final_code(STATUS_CODE_301_MOVED_PERMANENTLY)
        .await
        .expect("Error sending final response");

    assert_state_eq!(
        ctx.server_state,
        fsm::State::Completed,
        "must move to completed state after sending non_2xx final response"
    );

    ctx.client.send_ack_request().await;

    assert_state_eq!(
        ctx.server_state,
        fsm::State::Confirmed,
        "must move to confirmed state after receive ack message"
    );
}

#[tokio::test]
async fn unreliable_transition_to_terminated_immediately_when_receiving_2xx_response() {
    let mut ctx = UnreliableTransportTestContext::setup(SipMethod::Invite);

    ctx.server
        .respond_final_code(STATUS_CODE_202_ACCEPTED)
        .await
        .expect("should send final response with the provided code");

    assert_state_eq!(
        ctx.server_state,
        fsm::State::Terminated,
        "must terminate immediately when sending final 2xx response with invite transaction"
    );
}

#[tokio::test]
async fn reliable_transition_to_terminated_immediately_after_2xx_from_tu() {
    let mut ctx = ReliableTransportTestContext::setup(SipMethod::Invite);

    ctx.server
        .respond_final_code(STATUS_CODE_202_ACCEPTED)
        .await
        .expect("should send final response with the provided code");

    assert_state_eq!(
        ctx.server_state,
        fsm::State::Terminated,
        "must terminate immediately when sending final 2xx response with invite transaction"
    );
}

#[tokio::test]
async fn server_must_retransmit_final_non_2xx_response() {
    let ctx = RetransmissionTestContext::setup(SipMethod::Invite);
    let expected_responses = 1;
    let expected_retrans = 3;

    ctx.server
        .respond_final_code(STATUS_CODE_301_MOVED_PERMANENTLY)
        .await
        .expect("Error sending final response");

    ctx.client.retransmit_n_times(expected_retrans).await;

    assert_eq!(
        ctx.transport.sent_count(),
        expected_responses + expected_retrans
    );
}

#[tokio::test(start_paused = true)]
async fn server_transaction_must_cease_retransmission_when_receive_ack() {
    let mut ctx = RetransmissionTestContext::setup(SipMethod::Invite);
    let expected_responses = 1;
    let expected_retrans = 2;

    ctx.server
        .respond_final_code(STATUS_CODE_301_MOVED_PERMANENTLY)
        .await
        .expect("Error sending final response");

    ctx.timer.wait_for_retransmissions(2).await;

    ctx.client.send_ack_request().await;

    ctx.timer.wait_for_retransmissions(2).await;

    assert_eq!(
        ctx.transport.sent_count(),
        expected_responses + expected_retrans,
        "sent count should match {expected_responses} responses and {expected_retrans} retransmissions"
    );
}

#[tokio::test(start_paused = true)]
async fn timer_h_must_be_set_for_reliable_transports() {
    let mut ctx = ReliableTransportTestContext::setup(SipMethod::Invite);

    ctx.server
        .respond_final_code(STATUS_CODE_301_MOVED_PERMANENTLY)
        .await
        .expect("Error sending final response");

    assert_state_eq!(
        ctx.server_state,
        fsm::State::Completed,
        "transaction must not terminate immediately when unreliable transport is used"
    );

    tokio::time::sleep(crate::transaction::T1 * 64).await;

    assert_state_eq!(
        ctx.server_state,
        fsm::State::Terminated,
        "must terminate immediately when sending final non-2xx response with reliable transport"
    );
}

#[tokio::test(start_paused = true)]
async fn timer_h_must_be_set_for_unreliable_transports() {
    let mut ctx = UnreliableTransportTestContext::setup(SipMethod::Invite);

    ctx.server
        .respond_final_code(STATUS_CODE_301_MOVED_PERMANENTLY)
        .await
        .expect("Error sending final response");

    assert_state_eq!(
        ctx.server_state,
        fsm::State::Completed,
        "transaction must not terminate immediately when unreliable transport is used"
    );

    tokio::time::sleep(crate::transaction::T1 * 64).await;

    assert_state_eq!(
        ctx.server_state,
        fsm::State::Terminated,
        "must terminate immediately when sending final non-2xx response with reliable transport"
    );
}

#[tokio::test(start_paused = true)]
async fn test_timer_g_for_server_transaction() {
    let mut ctx = RetransmissionTestContext::setup(SipMethod::Invite);
    let expected_responses = 1;
    let expected_retrans = 5;

    ctx.server
        .respond_final_code(STATUS_CODE_301_MOVED_PERMANENTLY)
        .await
        .expect("Error sending final response");

    ctx.timer.wait_for_retransmissions(5).await;

    assert_eq!(
        ctx.transport.sent_count(),
        expected_responses + expected_retrans,
        "sent count should match {expected_responses} requests and {expected_retrans} retransmissions"
    );
}
