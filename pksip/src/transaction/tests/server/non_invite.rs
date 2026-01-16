use crate::{
    SipMethod, assert_state_eq,
    test_utils::TestContext,
    transaction::{
        fsm,
        tests::{STATUS_CODE_100_TRYING, STATUS_CODE_202_ACCEPTED, STATUS_CODE_504_SERVER_TIMEOUT},
    },
};

use super::{
    ReliableTransportTestContext, RetransmissionTestContext, UnreliableTransportTestContext,
};

#[tokio::test]
async fn transition_to_proceeding_after_1xx_from_tu() {
    let mut ctx = ReliableTransportTestContext::setup(SipMethod::Options);

    ctx.server
        .respond_provisional_code(STATUS_CODE_100_TRYING)
        .await
        .expect("transaction should send provisional response with the provided code");

    assert_state_eq!(
        ctx.server_state,
        fsm::State::Proceeding,
        "should move to proceeding state when sending provisional response"
    );
}

#[tokio::test]
async fn transition_to_completed_after_non_2xx_final_response_from_tu() {
    let mut ctx = UnreliableTransportTestContext::setup(SipMethod::Options);

    ctx.server
        .respond_final_code(STATUS_CODE_504_SERVER_TIMEOUT)
        .await
        .expect("should send final response with the provided code");

    assert_state_eq!(
        ctx.server_state,
        fsm::State::Completed,
        "must move to completed after receive 200-699 from TU"
    );
}

#[tokio::test]
async fn reliable_transition_to_terminated_immediately_after_2xx_from_tu() {
    let mut ctx = ReliableTransportTestContext::setup(SipMethod::Options);

    ctx.server
        .respond_final_code(STATUS_CODE_202_ACCEPTED)
        .await
        .expect("transaction should send final response with the provided code");

    assert_state_eq!(
        ctx.server_state,
        fsm::State::Terminated,
        "must terminate immediately when sending final 2xx response with reliable transport"
    );
}

#[tokio::test]
async fn reliable_transition_to_terminated_immediately_after_non_2xx_from_tu() {
    let mut ctx = ReliableTransportTestContext::setup(SipMethod::Options);

    ctx.server
        .respond_final_code(STATUS_CODE_504_SERVER_TIMEOUT)
        .await
        .expect("transaction should send final response with the provided code");

    assert_state_eq!(
        ctx.server_state,
        fsm::State::Terminated,
        "must terminate immediately when sending final non-2xx response with reliable transport"
    );
}

#[tokio::test]
async fn absorbs_retransmission_in_initial_state() {
    let ctx = RetransmissionTestContext::setup(SipMethod::Options);
    let expected_retrans_count = 0;

    ctx.client.retransmit_n_times(2).await;

    assert_eq!(ctx.transport.sent_count(), expected_retrans_count);
}

#[tokio::test]
async fn retransmit_provisional_response_in_proceeding_state() {
    let mut ctx = RetransmissionTestContext::setup(SipMethod::Options);
    let expected_response_count = 1;
    let expected_retrans_count = 4;

    ctx.server
        .respond_provisional_code(STATUS_CODE_100_TRYING)
        .await
        .expect("transaction should send provisional response with the provided code");

    ctx.client.retransmit_n_times(expected_retrans_count).await;

    assert_eq!(
        ctx.transport.sent_count(),
        expected_response_count + expected_retrans_count
    );
}

#[tokio::test]
async fn server_must_retransmit_final_2xx_response() {
    let ctx = RetransmissionTestContext::setup(SipMethod::Register);
    let expected_response_count = 1;
    let expected_retrans_count = 2;

    ctx.server
        .respond_final_code(STATUS_CODE_202_ACCEPTED)
        .await
        .expect("transaction should send final response with the provided code");

    ctx.client.retransmit_n_times(expected_retrans_count).await;

    assert_eq!(
        ctx.transport.sent_count(),
        expected_response_count + expected_retrans_count
    );
}

#[tokio::test(start_paused = true)]
async fn timer_j() {
    let mut ctx = UnreliableTransportTestContext::setup(SipMethod::Bye);

    ctx.server
        .respond_final_code(STATUS_CODE_202_ACCEPTED)
        .await
        .expect("transaction should send final response with the provided code");

    assert_state_eq!(
        ctx.server_state,
        fsm::State::Completed,
        "transaction must not terminate immediately when unreliable transport is used"
    );

    tokio::time::sleep(crate::transaction::T1 * 64).await;

    assert_state_eq!(
        ctx.server_state,
        fsm::State::Terminated,
        "must terminate after timer j fires"
    );
}
