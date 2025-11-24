use crate::{
    SipMethod,
    message::HostPort,
    transaction::{sip_transaction::Role},
    transport::IncomingMessageInfo,
};

/// Branch parameter prefix defined in RFC3261.
const RFC3261_BRANCH_ID: &str = "z9hG4bK";

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum TransactionKey {
    Rfc2543(Rfc2543),
    Rfc3261(Rfc3261),
}

impl TransactionKey {
    pub fn from_incoming(info: &IncomingMessageInfo) -> Self {
        match info.mandatory_headers.via.branch {
            Some(ref branch) if branch.starts_with(RFC3261_BRANCH_ID) => {
                let branch = branch.clone();
                let method = info.mandatory_headers.cseq.method;

                Self::new_key_3261(Role::UAS, method, branch)
            }
            _ => {
                todo!("create rfc 2543")
            }
        }
    }

    pub fn new_key_3261(role: Role, method: SipMethod, branch: String) -> Self {
        let method = if matches!(method, SipMethod::Invite | SipMethod::Ack) {
            None
        } else {
            Some(method)
        };

        Self::Rfc3261(Rfc3261 {
            role,
            branch,
            method,
        })
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Rfc2543 {
    pub cseq: u32,
    pub from_tag: Option<String>,
    pub to_tag: Option<String>,
    pub call_id: String,
    pub via_host_port: HostPort,
    pub method: Option<SipMethod>,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Rfc3261 {
    role: Role,
    branch: String,
    method: Option<SipMethod>,
}
