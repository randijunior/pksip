use tokio::sync::watch;

use crate::{
    SipMethod,
    transaction::{ServerTransaction, fsm},
    transport::Transport,
};

use crate::test_utils::{
    transaction::{
        MockClientTransaction, TestContext, TestRetransmissionTimer, create_test_endpoint,
        create_test_request,
    },
    transport::MockTransport,
};

mod invite;
mod non_invite;

struct RetransmissionTestContext {
    server: ServerTransaction,
    client: MockClientTransaction,
    transport: MockTransport,
    timer: TestRetransmissionTimer,
    server_state: watch::Receiver<fsm::State>,
}

impl TestContext for RetransmissionTestContext {
    fn setup(method: SipMethod) -> Self {
        let transport = MockTransport::new_udp();
        let transport_clone = transport.clone();

        let request = create_test_request(method, Transport::new(transport_clone));
        let endpoint = create_test_endpoint();
        let mut server = ServerTransaction::from_request(request.clone(), &endpoint).unwrap();

        let sender = endpoint
            .transactions()
            .get_entry(server.transaction_key())
            .unwrap();

        let client = MockClientTransaction { sender, request };

        let timer = TestRetransmissionTimer::new();

        let server_state = server.state_machine_mut().subscribe_state();

        RetransmissionTestContext {
            server,
            client,
            transport,
            timer,
            server_state,
        }
    }
}

struct UnreliableTransportTestContext {
    server: ServerTransaction,
    server_state: watch::Receiver<fsm::State>,
}

impl TestContext for UnreliableTransportTestContext {
    fn setup(method: SipMethod) -> Self {
        let (server, server_state) = setup_test_state(method, MockTransport::new_udp());

        Self {
            server,
            server_state,
        }
    }
}

struct ReliableTransportTestContext {
    server: ServerTransaction,
    server_state: watch::Receiver<fsm::State>,
}

impl TestContext for ReliableTransportTestContext {
    fn setup(method: SipMethod) -> Self {
        let (server, server_state) = setup_test_state(method, MockTransport::new_tcp());

        Self {
            server,
            server_state,
        }
    }
}

fn setup_test_state(
    method: SipMethod,
    transport: MockTransport,
) -> (ServerTransaction, watch::Receiver<fsm::State>) {
    let request = create_test_request(method, Transport::new(transport));
    let endpoint = create_test_endpoint();

    let mut server = ServerTransaction::from_request(request, &endpoint).unwrap();
    let state = server.state_machine_mut().subscribe_state();

    (server, state)
}
