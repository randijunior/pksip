use crate::{
    Result,
    dialog::{Dialog, DialogUsage},
    transaction::Role,
    transport::incoming::IncomingRequest,
};

enum SessionState {
    Inital,
    Calling,
    Incoming,
    Early,
    Connecting,
    Confirmed,
    Disconnected,
}

struct InviteSession {
    role: Role,
    dialog: Dialog,
    state: SessionState,
}

impl InviteSession {
    pub fn create_uas(dialog: Dialog) -> Self {
        Self {
            dialog,
            role: Role::UAS,
            state: SessionState::Inital,
        }
    }
}

#[async_trait::async_trait]
impl DialogUsage for InviteSession {
    async fn on_receive(&self, request: &mut Option<IncomingRequest>) -> Result<()> {
        Ok(())
    }
}
