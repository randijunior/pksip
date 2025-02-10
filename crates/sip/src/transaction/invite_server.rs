use std::{
    cmp, io, net::SocketAddr, ops::{Deref, DerefMut}
};

use async_trait::async_trait;
use tokio::{
    pin,
    sync::oneshot,
    time::{self, Instant},
};

use crate::{
    message::SipMethod, transaction::T2, transport::{ReceivedRequest, Transport},
};

use super::{
    SipTransaction, Transaction,TsxMsg,
    TsxState, TsxStateMachine, T1, T4,
};

pub struct ServerInviteTsx(Transaction);

impl ServerInviteTsx {
    pub fn new(
        addr: SocketAddr,
        transport: Transport
    ) -> Self {
        Self(Transaction {
            //When a server transaction is constructed for a request,
            // it enters the "Proceeding" state.
            state: TsxStateMachine::new(TsxState::Proceeding),
            addr,
            transport,
            last_msg: None,
            tx: None,
        })
    }
}

//The TU passes any number of provisional responses to the server
// transaction.

#[async_trait]
impl SipTransaction for ServerInviteTsx {
    async fn receive_message(
        &mut self,
        msg: TsxMsg,
    ) -> io::Result<()> {
        let state = self.get_state();
        match msg {
            TsxMsg::Request(req) => {
                /*
                 * If a request retransmission is received while in the
                 * "Proceeding" state, the most recent provisional response
                 * that was received from the TU MUST be passed
                 * to the transport layer for retransmission.
                 */
                if let TsxState::Proceeding = state {
                    self.retransmit().await?;
                }

                if req.is_method(&SipMethod::Ack)
                    && state == TsxState::Completed
                    && !self.reliable()
                {
                    self.state.confirmed();
                    self.do_terminate(T4);
                }
                return Ok(());
            }
            TsxMsg::Response(response) => {
                let code = response.code_num();
                if response.is_provisional() {
                    self.send(response).await?;
                    return Ok(());
                }
                // If, while in the "Proceeding" state, the TU passes a 2xx response to
                // the server transaction, the server transaction MUST pass this
                // response to the transport layer for transmission.
                // The server transaction MUST then transition to the "Terminated" state.
                if let TsxState::Proceeding = state {
                    self.send(response).await?;
                    match code {
                        200..=299 => {
                            self.state.terminated();
                        }
                        300..=699 => {
                            self.state.completed();
                            let buf = self
                                .last_msg
                                .as_ref()
                                .unwrap()
                                .buf
                                .as_ref()
                                .unwrap()
                                .clone();
                            let transport = self.transport.clone();
                            let addr = self.addr;
                            let redable = self.reliable();
                            let sender = self.tx.take().unwrap();
                            let state = self.state.clone();

                            tokio::spawn(async move {
                                pin! {
                                    let timer_g = time::sleep(T1);
                                    let timer_h = time::sleep(64*T1);
                                }
                                tokio::select! {
                                    _ = &mut timer_g => {
                                        if !redable && !matches!(state.get_state(), TsxState::Confirmed) {
                                            let _ = transport.send(&buf, addr);
                                            let next_interval = cmp::min(T1*2, T2);
                                            timer_g.reset(Instant::now() + next_interval);
                                        }
                                    }
                                    _= timer_h => {
                                        println!("Timer H Expired!");
                                        sender.send(()).unwrap();
                                        return;
                                    }
                                }
                            });
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }
        Ok(())
    }
}

impl Deref for ServerInviteTsx {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ServerInviteTsx {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

