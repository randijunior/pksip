use std::net::SocketAddr;

use tokio::sync::watch;

use crate::test_utils::{
    transaction::{
        MockServerTransaction, TestContext, TestRetransmissionTimer, create_test_endpoint,
        create_test_request,
    },
    transport::MockTransport,
};

use crate::{
    Endpoint,
    message::{Request, SipMethod},
    transaction::{ClientTransaction, fsm},
    transport::Transport,
};

mod invite;
mod non_invite;

struct SendRequestTestContext {
    endpoint: Endpoint,
    request: Request,
    target: (Transport, SocketAddr),
}

impl TestContext for SendRequestTestContext {
    fn setup(method: SipMethod) -> Self {
        let udp = MockTransport::new_udp();

        let transport = Transport::new(udp.clone());
        let request = create_test_request(method, transport.clone());

        let endpoint = create_test_endpoint();

        let target = (transport, request.info.transport.packet.source);

        Self {
            endpoint,
            request: request.message,
            target,
        }
    }
}

struct ReceiveProvisionalTestContext {
    server: MockServerTransaction,
    client: ClientTransaction,
}

impl TestContext for ReceiveProvisionalTestContext {
    async fn setup_async(method: SipMethod) -> Self {
        let transport = Transport::new(MockTransport::new_udp());
        let request = create_test_request(method, transport.clone());

        let endpoint = create_test_endpoint();

        let target = (transport, request.info.transport.packet.source);

        let client =
            ClientTransaction::send_request(&endpoint, request.message.clone(), Some(target))
                .await
                .expect("failure sending request");

        let sender = endpoint
            .transactions()
            .get_entry(client.transaction_key())
            .unwrap();

        let server = MockServerTransaction {
            sender,
            request,
            endpoint,
        };

        Self { server, client }
    }
}

struct ReceiveFinalTestContext {
    server: MockServerTransaction,
    client: ClientTransaction,
    client_state: watch::Receiver<fsm::State>,
}

impl TestContext for ReceiveFinalTestContext {
    async fn setup_async(method: SipMethod) -> Self {
        let transport = Transport::new(MockTransport::new_udp());
        let request = create_test_request(method, transport.clone());

        let endpoint = create_test_endpoint();

        let target = (transport, request.info.transport.packet.source);

        let mut client =
            ClientTransaction::send_request(&endpoint, request.message.clone(), Some(target))
                .await
                .expect("failure sending request");

        let sender = endpoint
            .transactions()
            .get_entry(client.transaction_key())
            .unwrap();

        let server = MockServerTransaction {
            sender,
            request,
            endpoint,
        };

        let client_state = client.state_machine_mut().subscribe_state();

        Self {
            server,
            client,
            client_state,
        }
    }
}

struct ReliableTransportTestContext {
    client: ClientTransaction,
    transport: MockTransport,
}

impl TestContext for ReliableTransportTestContext {
    async fn setup_async(method: SipMethod) -> Self {
        let tcp = MockTransport::new_tcp();

        let transport = Transport::new(tcp.clone());
        let request = create_test_request(method, transport.clone());

        let endpoint = create_test_endpoint();

        let target = (transport, request.info.transport.packet.source);

        let client =
            ClientTransaction::send_request(&endpoint, request.message.clone(), Some(target))
                .await
                .unwrap();

        Self {
            transport: tcp,
            client,
        }
    }
}

struct RetransmissionTestContext {
    server: MockServerTransaction,
    client: ClientTransaction,
    transport: MockTransport,
    timer: TestRetransmissionTimer,
}

impl TestContext for RetransmissionTestContext {
    async fn setup_async(method: SipMethod) -> Self {
        let timer = TestRetransmissionTimer::new();
        let udp = MockTransport::new_udp();

        let transport = Transport::new(udp.clone());
        let request = create_test_request(method, transport.clone());

        let endpoint = create_test_endpoint();

        let target = (transport, request.info.transport.packet.source);

        let client =
            ClientTransaction::send_request(&endpoint, request.message.clone(), Some(target))
                .await
                .unwrap();

        let expected_state = if method == SipMethod::Invite {
            fsm::State::Calling
        } else {
            fsm::State::Trying
        };

        assert_eq!(
            client.state(),
            expected_state,
            "Transaction state should transition to {expected_state} after sending request"
        );

        let sender = endpoint
            .transactions()
            .get_entry(client.transaction_key())
            .unwrap();

        let server = MockServerTransaction {
            sender,
            request,
            endpoint,
        };

        Self {
            client,
            server,
            transport: udp,
            timer,
        }
    }
}
