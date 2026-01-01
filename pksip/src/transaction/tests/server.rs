use crate::{
    assert_state_eq,
    message::Method,
    transaction::{
        T1, T2,
        TransactionState::{self},
    },
};

use super::{
    setup_test_server_receive_ack, setup_test_server_retransmission,
    setup_test_server_state_reliable, setup_test_server_state_unreliable,
};

const FINAL_2XX_STATUS_CODE: u16 = 202;
const FINAL_NON_2XX_STATUS_CODE: u16 = 301;
const PROVISIONAL_1XX_STATUS_CODE: u16 = 182;

// 17.2.1 INVITE Server Transaction
//
//                                |INVITE
//                                |pass INV to TU
//             INVITE             V send 100 if TU won't in 200ms
//             send response+-----------+
//                 +--------|           |--------+101-199 from TU
//                 |        | Proceeding|        |send response
//                 +------->|           |<-------+
//                          |           |          Transport Err.
//                          |           |          Inform TU
//                          |           |--------------->+
//                          +-----------+                |
//             300-699 from TU |     |2xx from TU        |
//             send response   |     |send response      |
//                             |     +------------------>+
//                             |                         |
//             INVITE          V          Timer G fires  |
//             send response+-----------+ send response  |
//                 +--------|           |--------+       |
//                 |        | Completed |        |       |
//                 +------->|           |<-------+       |
//                          +-----------+                |
//                             |     |                   |
//                         ACK |     |                   |
//                         -   |     +------------------>+
//                             |        Timer H fires    |
//                             V        or Transport Err.|
//                          +-----------+  Inform TU     |
//                          |           |                |
//                          | Confirmed |                |
//                          |           |                |
//                          +-----------+                |
//                                |                      |
//                                |Timer I fires         |
//                                |-                     |
//                                |                      |
//                                V                      |
//                          +-----------+                |
//                          |           |                |
//                          | Terminated|<---------------+
//                          |           |
//                          +-----------+

// 17.2.2 Non-INVITE Server Transaction
//
//                                |Request received
//                                |pass to TU
//                                V
//                          +-----------+
//                          |           |
//                          | Trying    |-------------+
//                          |           |             |
//                          +-----------+             |200-699 from TU
//                                |                   |send response
//                                |1xx from TU        |
//                                |send response      |
//                                |                   |
//             Request            V      1xx from TU  |
//             send response+-----------+send response|
//                 +--------|           |--------+    |
//                 |        | Proceeding|        |    |
//                 +------->|           |<-------+    |
//          +<--------------|           |             |
//          |Trnsprt Err    +-----------+             |
//          |Inform TU            |                   |
//          |                     |                   |
//          |                     |200-699 from TU    |
//          |                     |send response      |
//          |  Request            V                   |
//          |  send response+-----------+             |
//          |      +--------|           |             |
//          |      |        | Completed |<------------+
//          |      +------->|           |
//          +<--------------|           |
//          |Trnsprt Err    +-----------+
//          |Inform TU            |
//          |                     |Timer J fires
//          |                     |-
//          |                     |
//          |                     V
//          |               +-----------+
//          |               |           |
//          +-------------->| Terminated|
//                          |           |
//                          +-----------+


// ===== transaction state tests =====

#[tokio::test]
async fn non_invite_transition_to_proceeding_after_1xx_from_tu() {
    let (mut server_tsx, mut tsx_state) = setup_test_server_state_reliable(Method::Options);

    server_tsx
        .respond_with_provisional_code(PROVISIONAL_1XX_STATUS_CODE)
        .await
        .expect("transaction should send provisional response with the provided code");

    assert_state_eq!(
        tsx_state,
        TransactionState::Proceeding,
        "transaction should move to proceeding state when sending provisional response"
    );
}

#[tokio::test]
async fn non_invite_transition_to_completed_after_non_2xx_final_response_from_tu() {
    let (server_tsx, mut tsx_state) = setup_test_server_state_unreliable(Method::Options);

    server_tsx
        .respond_with_final_code(FINAL_NON_2XX_STATUS_CODE)
        .await
        .expect("transaction should send final response with the provided code");

    assert_state_eq!(
        tsx_state,
        TransactionState::Completed,
        "transaction must move to completed after receive 200-699 from TU"
    );
}

#[tokio::test]
async fn invite_transition_to_confirmed_state_after_receive_ack() {
    let (mut channel, mut tsx_state, server_tsx) = setup_test_server_receive_ack();

    server_tsx
        .respond_with_final_code(FINAL_NON_2XX_STATUS_CODE)
        .await
        .expect("transaction should send final response with the provided code");

    assert_state_eq!(
        tsx_state,
        TransactionState::Completed,
        "transaction must move to completed state after sending non_2xx final response"
    );

    channel.send_ack_request().await;

    assert_state_eq!(
        tsx_state,
        TransactionState::Confirmed,
        "transaction must move to confirmed state after receive ack message"
    );
}

#[tokio::test]
async fn non_invite_reliable_transition_to_terminated_immediately_after_2xx_from_tu() {
    let (server_tsx, mut tsx_state) = setup_test_server_state_reliable(Method::Options);

    server_tsx
        .respond_with_final_code(FINAL_2XX_STATUS_CODE)
        .await
        .expect("transaction should send final response with the provided code");

    assert_state_eq!(
        tsx_state,
        TransactionState::Terminated,
        "must terminate immediately when sending final 2xx response with reliable transport"
    );
}

#[tokio::test]
async fn non_invite_reliable_transition_to_terminated_immediately_after_non_2xx_from_tu() {
    let (server_tsx, mut tsx_state) = setup_test_server_state_reliable(Method::Options);

    server_tsx
        .respond_with_final_code(FINAL_NON_2XX_STATUS_CODE)
        .await
        .expect("transaction should send final response with the provided code");

    assert_state_eq!(
        tsx_state,
        TransactionState::Terminated,
        "must terminate immediately when sending final non-2xx response with reliable transport"
    );
}

#[tokio::test]
async fn invite_unreliable_transition_to_terminated_immediately_after_2xx_final_response_from_tu() {
    let (server_tsx, mut tsx_state) = setup_test_server_state_unreliable(Method::Invite);

    server_tsx
        .respond_with_final_code(FINAL_2XX_STATUS_CODE)
        .await
        .expect("transaction should send final response with the provided code");

    assert_state_eq!(
        tsx_state,
        TransactionState::Terminated,
        "must terminate immediately when sending final 2xx response with invite transaction"
    );
}

#[tokio::test]
async fn invite_reliable_transition_to_terminated_immediately_after_2xx_from_tu() {
    let (server_tsx, mut tsx_state) = setup_test_server_state_reliable(Method::Invite);

    server_tsx
        .respond_with_final_code(FINAL_2XX_STATUS_CODE)
        .await
        .expect("transaction should send final response with the provided code");

    assert_state_eq!(
        tsx_state,
        TransactionState::Terminated,
        "must terminate immediately when sending final 2xx response with invite transaction"
    );
}

// ===== transaction retransmission tests =====

#[tokio::test]
async fn absorbs_retransmission_in_initial_state() {
    let (channel, transport, _server_tsx) = setup_test_server_retransmission(Method::Options);
    let expected_retransmission_count = 0;

    channel.retransmit_n_times(2).await;

    assert_eq!(transport.sent_count(), expected_retransmission_count);
}

#[tokio::test]
async fn retransmit_provisional_response_in_proceeding_state() {
    let (channel, transport, mut server) = setup_test_server_retransmission(Method::Options);
    let expected_response_count = 1;
    let expected_retransmission_count = 4;

    server
        .respond_with_provisional_code(PROVISIONAL_1XX_STATUS_CODE)
        .await
        .expect("transaction should send provisional response with the provided code");

    channel
        .retransmit_n_times(expected_retransmission_count)
        .await;

    assert_eq!(
        transport.sent_count(),
        expected_response_count + expected_retransmission_count
    );
}

#[tokio::test]
async fn server_non_invite_must_retransmit_final_2xx_response() {
    let (channel, transport, server_tsx) = setup_test_server_retransmission(Method::Register);
    let expected_response_count = 1;
    let expected_retransmission_count = 2;

    server_tsx
        .respond_with_final_code(FINAL_2XX_STATUS_CODE)
        .await
        .expect("transaction should send final response with the provided code");

    channel
        .retransmit_n_times(expected_retransmission_count)
        .await;

    assert_eq!(
        transport.sent_count(),
        expected_response_count + expected_retransmission_count
    );
}

#[tokio::test]
async fn server_invite_must_retransmit_final_non_2xx_response() {
    let (channel, transport, server_tsx) = setup_test_server_retransmission(Method::Invite);
    let expected_response_count = 1;
    let expected_retransmission_count = 3;

    server_tsx
        .respond_with_final_code(FINAL_NON_2XX_STATUS_CODE)
        .await
        .expect("transaction should send final response with the provided code");

    channel
        .retransmit_n_times(expected_retransmission_count)
        .await;

    assert_eq!(
        transport.sent_count(),
        expected_response_count + expected_retransmission_count
    );
}

#[tokio::test(start_paused = true)]
async fn server_transaction_must_cease_retransmission_when_receive_ack() {
    let (mut channel, transport, server_tsx) = setup_test_server_retransmission(Method::Invite);
    let expected_response_count = 1;
    let expected_retransmission_count = 2;

    server_tsx
        .respond_with_final_code(FINAL_NON_2XX_STATUS_CODE)
        .await
        .expect("transaction should send final response with the provided code");

    let mut interval = T1;
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
        expected_response_count + expected_retransmission_count
    );
}

// ===== transaction timers tests =====

#[tokio::test(start_paused = true)]
async fn timer_h_must_be_set_for_reliable_transports() {
    let (server_tsx, mut tsx_state) = setup_test_server_state_reliable(Method::Invite);

    server_tsx
        .respond_with_final_code(FINAL_NON_2XX_STATUS_CODE)
        .await
        .expect("transaction should send final response with the provided code");

    assert_state_eq!(
        tsx_state,
        TransactionState::Completed,
        "transaction must not terminate immediately when unreliable transport is used"
    );

    tokio::time::sleep(T1 * 64).await;

    assert_state_eq!(
        tsx_state,
        TransactionState::Terminated,
        "must terminate immediately when sending final non-2xx response with reliable transport"
    );
}

#[tokio::test(start_paused = true)]
async fn timer_h_must_be_set_for_unreliable_transports() {
    let (server_tsx, mut tsx_state) = setup_test_server_state_unreliable(Method::Invite);

    server_tsx
        .respond_with_final_code(FINAL_NON_2XX_STATUS_CODE)
        .await
        .expect("transaction should send final response with the provided code");

    assert_state_eq!(
        tsx_state,
        TransactionState::Completed,
        "transaction must not terminate immediately when unreliable transport is used"
    );

    tokio::time::sleep(T1 * 64).await;

    assert_state_eq!(
        tsx_state,
        TransactionState::Terminated,
        "must terminate immediately when sending final non-2xx response with reliable transport"
    );
}

#[tokio::test(start_paused = true)]
async fn test_timer_g_for_invite_server_transaction() {
    let (_channel, transport, server_tsx) = setup_test_server_retransmission(Method::Invite);
    let expected_response_count = 1;
    let expected_retransmission_count = 5;

    server_tsx
        .respond_with_final_code(FINAL_NON_2XX_STATUS_CODE)
        .await
        .expect("transaction should send final response with the provided code");

    let mut interval = T1;
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
        expected_response_count + expected_retransmission_count
    );
}

#[tokio::test(start_paused = true)]
async fn test_timer_j_for_non_invite_server_transaction() {
    let (server_tsx, mut tsx_state) = setup_test_server_state_unreliable(Method::Bye);

    server_tsx
        .respond_with_final_code(FINAL_2XX_STATUS_CODE)
        .await
        .expect("transaction should send final response with the provided code");

    assert_state_eq!(
        tsx_state,
        TransactionState::Completed,
        "transaction must not terminate immediately when unreliable transport is used"
    );

    tokio::time::sleep(T1 * 64).await;

    assert_state_eq!(
        tsx_state,
        TransactionState::Terminated,
        "must terminate immediately when sending final non-2xx response with reliable transport"
    );
}
