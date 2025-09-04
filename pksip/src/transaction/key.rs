use crate::header::Header;
use crate::message::HostPort;
use crate::message::SipMethod;
use crate::transport::IncomingRequest;
use crate::transport::OutgoingRequest;
use crate::ArcStr;

const BRANCH_MAGIC_COOKIE: &str = "z9hG4bK";

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum TransactionKey {
    Rfc2543(Rfc2543),
    Rfc3261(Rfc3261),
}

impl TransactionKey {
    pub fn create_client_with(method: &SipMethod, branch: &str) -> Self {
        TransactionKey::Rfc3261(Rfc3261::Client(ClientTransactionKey {
            branch: branch.into(),
            method: Some(*method),
        }))
    }

    pub fn create_client(request: &OutgoingRequest) -> Self {
        let via = request
            .msg
            .headers
            .iter()
            .filter_map(|header| match header {
                Header::Via(via_hdr) => Some(via_hdr),
                _ => None,
            })
            .next()
            .unwrap();

        let cseq = request
            .msg
            .headers
            .iter()
            .filter_map(|header| match header {
                Header::CSeq(cseq) => Some(cseq),
                _ => None,
            })
            .next()
            .unwrap();

        match via.branch() {
            Some(branch) => {
                // Valid branch for RFC 3261
                TransactionKey::Rfc3261(Rfc3261::Client(ClientTransactionKey {
                    branch: branch.into(),
                    method: Some(*cseq.method()),
                }))
            }
            _ => {
                todo!("Generate branch parameter if it doesn't exist");
            }
        }
    }

    pub fn create_server(request: &IncomingRequest) -> Self {
        match request.request_headers.via.branch() {
            Some(branch) if branch.starts_with(BRANCH_MAGIC_COOKIE) => {
                TransactionKey::Rfc3261(Rfc3261::Server(ServerTransactionKey {
                    branch: branch.into(),
                    via_sent_by: request.request_headers.via.sent_by().clone(),
                    method: Some(*request.request_headers.cseq.method()),
                }))
            }
            _ => {
                todo!("create rfc 2543")
            }
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Rfc2543 {
    pub cseq: u32,
    pub from_tag: Option<ArcStr>,
    pub to_tag: Option<ArcStr>,
    pub call_id: ArcStr,
    pub via_host_port: HostPort,
    pub method: Option<SipMethod>,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Rfc3261 {
    Client(ClientTransactionKey),
    Server(ServerTransactionKey),
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ClientTransactionKey {
    branch: ArcStr,
    method: Option<SipMethod>,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ServerTransactionKey {
    branch: ArcStr,
    via_sent_by: HostPort,
    method: Option<SipMethod>,
}
