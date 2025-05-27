use std::sync::Arc;

use crate::{
    message::{HostPort, Method},
    transport::{IncomingRequest, OutgoingRequest},
};

const BRANCH_RFC3261: &str = "z9hG4bK";

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum TsxKey {
    Rfc2543(Rfc2543),
    Rfc3261(Rfc3261),
}

impl TsxKey {

    pub fn create_client_with(method: &Method, branch: &str) -> Self {
        TsxKey::Rfc3261(Rfc3261::Client(ClientTsxKey {
            branch: branch.into(),
            method: Some(*method),
        }))
    }
    pub fn create_client(request: &OutgoingRequest) -> Self {
        let branch = request.req_headers.via.branch();

        match branch {
            Some(branch) => {
                // Valid branch for RFC 3261
                TsxKey::Rfc3261(Rfc3261::Client(ClientTsxKey {
                    branch: branch.into(),
                    method: Some(*request.req_headers.cseq.method()),
                }))
            }
            _ => {
                todo!("Generate branch parameter if it doesn't exist");
            }
        }
    }

    pub fn create_server(request: &IncomingRequest) -> Self {
        match request.req_headers.via.branch() {
            Some(branch) if branch.starts_with(BRANCH_RFC3261) => TsxKey::Rfc3261(Rfc3261::Server(ServerTsxKey {
                branch: branch.into(),
                via_sent_by: request.req_headers.via.sent_by().clone(),
                method: Some(*request.req_headers.cseq.method()),
            })),
            _ => {
                todo!("create rfc 2543")
            }
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Rfc2543 {
    pub cseq: u32,
    pub from_tag: Option<Box<str>>,
    pub to_tag: Option<Box<str>>,
    pub call_id: Box<str>,
    pub via_host_port: HostPort,
    pub method: Option<Method>,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Rfc3261 {
    Client(ClientTsxKey),
    Server(ServerTsxKey),
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ClientTsxKey {
    branch: Box<str>,
    method: Option<Method>,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ServerTsxKey {
    branch: Box<str>,
    via_sent_by: HostPort,
    method: Option<Method>,
}
