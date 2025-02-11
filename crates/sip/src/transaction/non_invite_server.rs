use async_trait::async_trait;

use crate::{transaction::TsxState, transport::Transport};
use std::{
    io,
    net::SocketAddr,
    ops::{Deref, DerefMut},
};

use super::{
    SipTransaction, Transaction, TsxMsg, TsxStateMachine, T1,
};

pub struct ServerNonInviteTsx(Transaction);

impl ServerNonInviteTsx {
    // The state machine is initialized in the "Trying" state and is passed
    // a request other than INVITE or ACK when initialized.
    pub fn new(addr: SocketAddr, transport: Transport) -> Self {
        Self(Transaction {
            state: TsxStateMachine::new(TsxState::Trying),
            addr,
            transport,
            last_msg: None,
            tx: None,
        })
    }
}

#[async_trait]
impl SipTransaction for ServerNonInviteTsx {
    async fn receive_message(
        &mut self,
        msg: TsxMsg,
    ) -> io::Result<()> {
        let state = self.get_state();
        if let TsxState::Completed = state {
            return Ok(());
        }
        let TsxMsg::Response(response) = msg else {
            if let TsxState::Trying = state {
                // Once in the "Trying" state, any further request
                // retransmissions are discarded.
                return Ok(());
            }
            if let TsxState::Proceeding = state {
                self.retransmit().await?;
            }
            return Ok(());
        };

        if response.is_provisional() {
            self.send(response).await?;
            if let TsxState::Trying = state {
                self.state.proceeding();
                return Ok(());
            }
        } else {
            self.send(response).await?;
            if matches!(
                state,
                TsxState::Proceeding | TsxState::Trying
            ) {
                self.state.completed();
                self.do_terminate(T1 * 64);
            }
        }
        Ok(())
    }
}

impl Deref for ServerNonInviteTsx {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ServerNonInviteTsx {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
