use crate::{
    Method, assert_state_eq,
    transaction::{T1, T2, fsm},
};

use super::{
    setup_test_server_receive_ack, setup_test_server_retransmission,
    setup_test_server_state_reliable, setup_test_server_state_unreliable,
};

// ===== transaction state tests =====

#[tokio::test]
async fn invite_transition_to_confirmed_state_after_receive_ack() {
    let (mut channel, mut tsx_state, server_tsx) = setup_test_server_receive_ack();

    server_tsx
        .respond_with_final_code(super::FINAL_NON_2XX_STATUS_CODE)
        .await
        .expect("transaction should send final response with the provided code");

    assert_state_eq!(
        tsx_state,
        fsm::State::Completed,
        "must move to completed state after sending non_2xx final response"
    );

    channel.send_ack_request().await;

    assert_state_eq!(
        tsx_state,
        fsm::State::Confirmed,
        "must move to confirmed state after receive ack message"
    );
}

#[tokio::test]
async fn invite_unreliable_transition_to_terminated_immediately_after_2xx_final_response_from_tu() {
    let (server_tsx, mut tsx_state) = setup_test_server_state_unreliable(Method::Invite);

    server_tsx
        .respond_with_final_code(super::FINAL_2XX_STATUS_CODE)
        .await
        .expect("should send final response with the provided code");

    assert_state_eq!(
        tsx_state,
        fsm::State::Terminated,
        "must terminate immediately when sending final 2xx response with invite transaction"
    );
}

#[tokio::test]
async fn invite_reliable_transition_to_terminated_immediately_after_2xx_from_tu() {
    let (server_tsx, mut tsx_state) = setup_test_server_state_reliable(Method::Invite);

    server_tsx
        .respond_with_final_code(super::FINAL_2XX_STATUS_CODE)
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
async fn server_invite_must_retransmit_final_non_2xx_response() {
    let (channel, transport, server_tsx) = setup_test_server_retransmission(Method::Invite);
    let expected_response_count = 1;
    let expected_retrans_count = 3;

    server_tsx
        .respond_with_final_code(super::FINAL_NON_2XX_STATUS_CODE)
        .await
        .expect("transaction should send final response with the provided code");

    channel.retransmit_n_times(expected_retrans_count).await;

    assert_eq!(
        transport.sent_count(),
        expected_response_count + expected_retrans_count
    );
}

#[tokio::test(start_paused = true)]
async fn server_transaction_must_cease_retransmission_when_receive_ack() {
    let (mut channel, transport, server_tsx) = setup_test_server_retransmission(Method::Invite);
    let expected_response_count = 1;
    let expected_retrans_count = 2;
    let mut interval = T1;

    server_tsx
        .respond_with_final_code(super::FINAL_NON_2XX_STATUS_CODE)
        .await
        .expect("transaction should send final response with the provided code");

    tokio::time::sleep(interval).await;
    interval *= 2;
    tokio::time::sleep(interval).await;
    interval *= 2;
    tokio::task::yield_now().await;

    channel.send_ack_request().await;

    tokio::time::sleep(interval).await;
    interval = T2;
    tokio::time::sleep(interval).await;
    
    tokio::task::yield_now().await;

    assert_eq!(
        transport.sent_count(),
        expected_response_count + expected_retrans_count
    );
}

// ===== transaction timers tests =====

#[tokio::test(start_paused = true)]
async fn timer_h_must_be_set_for_reliable_transports() {
    let (server_tsx, mut tsx_state) = setup_test_server_state_reliable(Method::Invite);

    server_tsx
        .respond_with_final_code(super::FINAL_NON_2XX_STATUS_CODE)
        .await
        .expect("transaction should send final response with the provided code");

    assert_state_eq!(
        tsx_state,
        fsm::State::Completed,
        "transaction must not terminate immediately when unreliable transport is used"
    );

    tokio::time::sleep(T1 * 64).await;

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
        .expect("transaction should send final response with the provided code");

    assert_state_eq!(
        tsx_state,
        fsm::State::Completed,
        "transaction must not terminate immediately when unreliable transport is used"
    );

    tokio::time::sleep(T1 * 64).await;

    assert_state_eq!(
        tsx_state,
        fsm::State::Terminated,
        "must terminate immediately when sending final non-2xx response with reliable transport"
    );
}

#[tokio::test(start_paused = true)]
async fn test_timer_g_for_invite_server_transaction() {
    let (_channel, transport, server_tsx) = setup_test_server_retransmission(Method::Invite);
    let expected_response_count = 1;
    let expected_retrans_count = 5;
    let mut interval = T1;

    server_tsx
        .respond_with_final_code(super::FINAL_NON_2XX_STATUS_CODE)
        .await
        .expect("transaction should send final response with the provided code");
    
    tokio::time::sleep(interval).await;
    interval *= 2;
    tokio::time::sleep(interval).await;
    interval *= 2;
    tokio::time::sleep(interval).await;
    interval *= 2;
    tokio::time::sleep(interval).await;
    interval = T2;
    tokio::time::sleep(interval).await;

    tokio::task::yield_now().await;

    assert_eq!(
        transport.sent_count(),
        expected_response_count + expected_retrans_count
    );
}
