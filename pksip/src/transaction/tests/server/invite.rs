use crate::{
    Method, assert_state_eq,
    transaction::{
        fsm,
        tests::{
            STATUS_CODE_202_ACCEPTED, STATUS_CODE_301_MOVED_PERMANENTLY, TestRetransmissionTimer,
        },
    },
};

use super::{
    setup_test_server_receive_ack, setup_test_server_retransmission,
    setup_test_server_state_reliable, setup_test_server_state_unreliable,
};

// ===== transaction state tests =====

#[tokio::test]
async fn transitions_to_confirmed_state_after_receive_ack() {
    let (mut client, mut state, server_tsx) = setup_test_server_receive_ack();

    server_tsx
        .respond_with_final_code(STATUS_CODE_301_MOVED_PERMANENTLY)
        .await
        .expect("Error sending final response");

    assert_state_eq!(
        state,
        fsm::State::Completed,
        "must move to completed state after sending non_2xx final response"
    );

    client.send_ack_request().await;

    assert_state_eq!(
        state,
        fsm::State::Confirmed,
        "must move to confirmed state after receive ack message"
    );
}

#[tokio::test]
async fn unreliable_transition_to_terminated_immediately_when_receiving_2xx_response() {
    let (server_tsx, mut tsx_state) = setup_test_server_state_unreliable(Method::Invite);

    server_tsx
        .respond_with_final_code(STATUS_CODE_202_ACCEPTED)
        .await
        .expect("should send final response with the provided code");

    assert_state_eq!(
        tsx_state,
        fsm::State::Terminated,
        "must terminate immediately when sending final 2xx response with invite transaction"
    );
}

#[tokio::test]
async fn reliable_transition_to_terminated_immediately_after_2xx_from_tu() {
    let (server_tsx, mut tsx_state) = setup_test_server_state_reliable(Method::Invite);

    server_tsx
        .respond_with_final_code(STATUS_CODE_202_ACCEPTED)
        .await
        .expect("should send final response with the provided code");

    assert_state_eq!(
        tsx_state,
        fsm::State::Terminated,
        "must terminate immediately when sending final 2xx response with invite transaction"
    );
}

// ===== transaction retransmission tests =====

#[tokio::test]
async fn server_must_retransmit_final_non_2xx_response() {
    let (client, transport, server_tsx) = setup_test_server_retransmission(Method::Invite);
    let expected_responses = 1;
    let expected_retrans = 3;

    server_tsx
        .respond_with_final_code(super::FINAL_NON_2XX_STATUS_CODE)
        .await
        .expect("Error sending final response");

    client.retransmit_n_times(expected_retrans).await;

    assert_eq!(
        transport.sent_count(),
        expected_responses + expected_retrans
    );
}

#[tokio::test(start_paused = true)]
async fn server_transaction_must_cease_retransmission_when_receive_ack() {
    let (mut client, transport, server_tsx) = setup_test_server_retransmission(Method::Invite);
    let mut timer = TestRetransmissionTimer::new();
    let expected_responses = 1;
    let expected_retrans = 2;

    server_tsx
        .respond_with_final_code(super::FINAL_NON_2XX_STATUS_CODE)
        .await
        .expect("Error sending final response");

    timer.wait_for_retransmissions(2).await;

    client.send_ack_request().await;

    timer.wait_for_retransmissions(2).await;

    assert_eq!(
        transport.sent_count(),
        expected_responses + expected_retrans,
        "sent count should match {expected_responses} responses and {expected_retrans} retransmissions"
    );
}

// ===== transaction timers tests =====

#[tokio::test(start_paused = true)]
async fn timer_h_must_be_set_for_reliable_transports() {
    let (server_tsx, mut tsx_state) = setup_test_server_state_reliable(Method::Invite);

    server_tsx
        .respond_with_final_code(super::FINAL_NON_2XX_STATUS_CODE)
        .await
        .expect("Error sending final response");

    assert_state_eq!(
        tsx_state,
        fsm::State::Completed,
        "transaction must not terminate immediately when unreliable transport is used"
    );

    tokio::time::sleep(crate::transaction::T1 * 64).await;

    assert_state_eq!(
        tsx_state,
        fsm::State::Terminated,
        "must terminate immediately when sending final non-2xx response with reliable transport"
    );
}

#[tokio::test(start_paused = true)]
async fn timer_h_must_be_set_for_unreliable_transports() {
    let (server_tsx, mut tsx_state) = setup_test_server_state_unreliable(Method::Invite);

    server_tsx
        .respond_with_final_code(super::FINAL_NON_2XX_STATUS_CODE)
        .await
        .expect("Error sending final response");

    assert_state_eq!(
        tsx_state,
        fsm::State::Completed,
        "transaction must not terminate immediately when unreliable transport is used"
    );

    tokio::time::sleep(crate::transaction::T1 * 64).await;

    assert_state_eq!(
        tsx_state,
        fsm::State::Terminated,
        "must terminate immediately when sending final non-2xx response with reliable transport"
    );
}

#[tokio::test(start_paused = true)]
async fn test_timer_g_for_server_transaction() {
    let (_client, transport, server_tsx) = setup_test_server_retransmission(Method::Invite);
    let mut timer = TestRetransmissionTimer::new();
    let expected_responses = 1;
    let expected_retrans = 5;

    server_tsx
        .respond_with_final_code(super::FINAL_NON_2XX_STATUS_CODE)
        .await
        .expect("Error sending final response");

    timer.wait_for_retransmissions(5).await;

    assert_eq!(
        transport.sent_count(),
        expected_responses + expected_retrans,
        "sent count should match {expected_responses} requests and {expected_retrans} retransmissions"
    );
}
