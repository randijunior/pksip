use crate::{
    headers::CallId,
    internal::ArcStr,
    message::{HostPort, SipMethod},
    transport::IncomingRequest,
};

const BRANCH_RFC3261: &str = "z9hG4bK";

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum TsxKey {
    Rfc2543(Rfc2543),
    Rfc3261(Rfc3261),
}

impl TsxKey {
    pub fn create(request: &IncomingRequest) -> Self {
        let hdrs = request.msg.req_headers.as_ref().unwrap();

        match &hdrs.via.branch {
            Some(branch) if branch.starts_with(BRANCH_RFC3261) => {
                TsxKey::Rfc3261(Rfc3261 {
                    branch: branch.clone(),
                    via_sent_by: hdrs.via.sent_by.clone(),
                    method: Some(hdrs.cseq.method),
                    cseq: hdrs.cseq.cseq,
                })
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
    pub call_id: CallId,
    pub via_host_port: HostPort,
    pub method: Option<SipMethod>,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Rfc3261 {
    branch: ArcStr,
    via_sent_by: HostPort,
    method: Option<SipMethod>,
    cseq: u32,
}
