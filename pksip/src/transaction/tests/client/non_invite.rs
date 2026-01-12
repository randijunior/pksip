use std::time::Duration;

use crate::{
    Method, assert_state_eq,
    transaction::{
        ClientTransaction,
        fsm::{self},
    },
};

use super::{
    setup_test_recv_final_response, setup_test_recv_provisional_response,
    setup_test_send_request,
};

#[tokio::test]
async fn transition_to_trying_after_request_sent() {
    let (endpoint, request, target) = setup_test_send_request(Method::Bye);

    let client = ClientTransaction::send_request(&endpoint, request, Some(target))
        .await
        .expect("failure sending request");

    assert_eq!(client.state(), fsm::State::Trying);
}

#[tokio::test]
async fn transition_to_proceeding_after_receive_provisional_response() {
    let (server, mut client) = setup_test_recv_provisional_response(Method::Register).await;

    server.respond(super::PROVISIONAL_1XX_STATUS_CODE).await;

    assert!(client.receive_provisional_response().await.is_ok());
    assert_eq!(client.state(), fsm::State::Proceeding);
}

#[tokio::test]
async fn transition_to_completed_after_receive_final_response() {
    let (server, client, mut state) = setup_test_recv_final_response(Method::Options).await;

    server.respond(super::FINAL_NON_2XX_STATUS_CODE).await;

    assert!(client.receive_final_response().await.is_ok());
    assert_state_eq!(state, fsm::State::Completed, "Unexpected state");
}

#[tokio::test(start_paused = true)]
async fn transition_to_terminated_after_timer_f_fires() {
    let (_server, mut client) = setup_test_recv_provisional_response(Method::Bye).await;

    tokio::time::advance(Duration::from_millis(500 * 64)).await;
    tokio::task::yield_now().await;

    assert!(client.receive_provisional_response().await.is_err());
    assert_eq!(client.state(), fsm::State::Terminated);
}
