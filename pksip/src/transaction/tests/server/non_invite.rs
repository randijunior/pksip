use crate::{
    Method, assert_state_eq,
    transaction::{T1, fsm},
};

use super::{
    setup_test_server_retransmission, setup_test_server_state_reliable,
    setup_test_server_state_unreliable,
};

#[tokio::test]
async fn transition_to_proceeding_after_1xx_from_tu() {
    let (mut server_tsx, mut tsx_state) = setup_test_server_state_reliable(Method::Options);

    server_tsx
        .respond_with_provisional_code(super::PROVISIONAL_1XX_STATUS_CODE)
        .await
        .expect("transaction should send provisional response with the provided code");

    assert_state_eq!(
        tsx_state,
        fsm::State::Proceeding,
        "should move to proceeding state when sending provisional response"
    );
}

#[tokio::test]
async fn transition_to_completed_after_non_2xx_final_response_from_tu() {
    let (server_tsx, mut tsx_state) = setup_test_server_state_unreliable(Method::Options);

    server_tsx
        .respond_with_final_code(super::FINAL_NON_2XX_STATUS_CODE)
        .await
        .expect("should send final response with the provided code");

    assert_state_eq!(
        tsx_state,
        fsm::State::Completed,
        "must move to completed after receive 200-699 from TU"
    );
}

#[tokio::test]
async fn reliable_transition_to_terminated_immediately_after_2xx_from_tu() {
    let (server_tsx, mut tsx_state) = setup_test_server_state_reliable(Method::Options);

    server_tsx
        .respond_with_final_code(super::FINAL_2XX_STATUS_CODE)
        .await
        .expect("transaction should send final response with the provided code");

    assert_state_eq!(
        tsx_state,
        fsm::State::Terminated,
        "must terminate immediately when sending final 2xx response with reliable transport"
    );
}

#[tokio::test]
async fn reliable_transition_to_terminated_immediately_after_non_2xx_from_tu() {
    let (server_tsx, mut tsx_state) = setup_test_server_state_reliable(Method::Options);

    server_tsx
        .respond_with_final_code(super::FINAL_NON_2XX_STATUS_CODE)
        .await
        .expect("transaction should send final response with the provided code");

    assert_state_eq!(
        tsx_state,
        fsm::State::Terminated,
        "must terminate immediately when sending final non-2xx response with reliable transport"
    );
}

#[tokio::test]
async fn absorbs_retransmission_in_initial_state() {
    let (channel, transport, _server_tsx) = setup_test_server_retransmission(Method::Options);
    let expected_retrans_count = 0;

    channel.retransmit_n_times(2).await;

    assert_eq!(transport.sent_count(), expected_retrans_count);
}

#[tokio::test]
async fn retransmit_provisional_response_in_proceeding_state() {
    let (channel, transport, mut server) = setup_test_server_retransmission(Method::Options);
    let expected_response_count = 1;
    let expected_retrans_count = 4;

    server
        .respond_with_provisional_code(super::PROVISIONAL_1XX_STATUS_CODE)
        .await
        .expect("transaction should send provisional response with the provided code");

    channel
        .retransmit_n_times(expected_retrans_count)
        .await;

    assert_eq!(
        transport.sent_count(),
        expected_response_count + expected_retrans_count
    );
}

#[tokio::test]
async fn server_must_retransmit_final_2xx_response() {
    let (channel, transport, server_tsx) = setup_test_server_retransmission(Method::Register);
    let expected_response_count = 1;
    let expected_retrans_count = 2;

    server_tsx
        .respond_with_final_code(super::FINAL_2XX_STATUS_CODE)
        .await
        .expect("transaction should send final response with the provided code");

    channel
        .retransmit_n_times(expected_retrans_count)
        .await;

    assert_eq!(
        transport.sent_count(),
        expected_response_count + expected_retrans_count
    );
}

#[tokio::test(start_paused = true)]
async fn timer_j() {
    let (server_tsx, mut tsx_state) = setup_test_server_state_unreliable(Method::Bye);

    server_tsx
        .respond_with_final_code(super::FINAL_2XX_STATUS_CODE)
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
        "must terminate after timer j fires"
    );
}
