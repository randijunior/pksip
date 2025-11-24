use std::{
    sync::{Arc, RwLock, atomic::AtomicU32},
    time::Duration,
};

use crate::{
    SipMethod,
    endpoint::Endpoint,
    error::Result,
    find_map_header,
    headers::Via,
    transaction::key::TransactionKey,
    transport::{IncomingRequest, OutgoingMessage, OutgoingRequest, OutgoingResponse},
};

use tokio::sync::RwLock as AsyncRwLock;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Defines the possible states of a SIP Transaction.
pub enum TransactionState {
    #[default]
    /// Initial state
    Initial,
    /// Calling state
    Calling,
    /// Trying state
    Trying,
    /// Proceeding state
    Proceeding,
    /// Completed state
    Completed,
    /// Confirmed state
    Confirmed,
    /// Terminated state
    Terminated,
}

/// The inner state of a [`Transaction`].
struct Inner {
    role: Role,
    method: SipMethod,
    key: TransactionKey,
    endpoint: Endpoint,
    state: RwLock<TransactionState>,
    retransmit_count: AtomicU32,
    // TransportDestinationInfo
}

#[derive(Clone)]
/// Represents a SIP Transaction.
/// 
/// Is used to handle both `UAS` and `UAC` transaction.
pub struct Transaction {
    inner: Arc<Inner>,
}

impl Transaction {
    pub(crate) fn new_client(request: &OutgoingRequest, endpoint: &Endpoint) -> Result<Self> {
        let message = &request.message;
        
        let via = find_map_header!(message.headers, Via);
        let Some(via) = via else {
            todo!("Via::new_with_transport(sent_by, branch)")
            // Via::new_with_transport(sent_by, branch)
        };
        let Some(branch) = via.branch.clone() else {
            todo!("Generate Branch")
        };

        let method = message.req_line.method;
        let key = TransactionKey::new_key_3261(Role::UAC, method, branch);
        let mut builder = Self::builder();

        builder.set_key(key);
        builder.set_role(Role::UAC);
        builder.set_method(method);
        builder.set_endpoint(endpoint.clone());

        Ok(builder.build())
    }
    
    pub(crate) fn create_server(request: &IncomingRequest, endpoint: &Endpoint) -> Result<Self> {
        let method = request.message.req_line.method;

        if method == SipMethod::Ack {
            todo!("Return Err")
        }

        let key = TransactionKey::from_incoming(&request.info);
        let mut builder = Self::builder();

        builder.set_key(key);
        builder.set_role(Role::UAS);
        builder.set_method(method);
        builder.set_endpoint(endpoint.clone());

        Ok(builder.build())
    }

    pub fn key(&self) -> &TransactionKey {
        &self.inner.key
    }

    fn builder() -> Builder {
        Default::default()
    }

    fn schedule_termination(&self, time: Duration) {
        let tsx = self.clone();
        tokio::spawn(async move {
            tokio::time::sleep(time).await;
            tsx.on_terminated();
        });
    }

    #[inline]
    /// Checks if the transport is reliable.
    pub fn is_reliable(&self) -> bool {
        todo!()
        // self.inner.transport.is_reliable()
    }

    #[inline]
    /// Retrieves the current state of the Transaction.
    pub fn get_state(&self) -> TransactionState {
        todo!()
        // self.inner
        //     .state
        //     .lock()
        //     .expect("Lock failed")
        //     .transaction_state
    }

    #[inline]
    /// Gets the count of retransmissions.
    pub fn retrans_count(&self) -> u32 {
        todo!()
        // self.inner.retrans_count.load(Ordering::SeqCst) as u32
    }

    #[inline]
    pub(crate) fn add_retrans_count(&self) -> u32 {
        todo!()
        // self.inner.retrans_count.fetch_add(1, Ordering::SeqCst) as u32 + 1
    }

    fn on_terminated(&self) {
        self.set_state(TransactionState::Terminated);

        // let layer = self.inner.registration.read().unwrap().unwrap().endpoint.transactions();
        // let key = &self.inner.registration.read().unwrap().unwrap().key;

        // match self.inner.role {
        //     Role::UAC => {
        //         layer.remove_client_tsx(key);
        //     }
        //     Role::UAS => {
        //         layer.remove_server_tsx(key);
        //     }
        // };
    }

    fn set_state(&self, state: TransactionState) {
        todo!()
        // let old = {
        //     let mut guard = self.inner.state.lock().expect("Lock failed");
        //     mem::replace(&mut *&mut guard.transaction_state, state)
        // };
        // log::trace!("State Changed [{old:?} -> {state:?}] ({:p})", self.inner);
    }

    pub(crate) fn is_calling(&self) -> bool {
        self.get_state() == TransactionState::Calling
    }

    async fn retransmit(&self) -> Result<u32> {
        todo!()
        // let retransmited = {
        //     let lock = self.inner.last_msg.read().await;
        //     if let Some(msg) = lock.as_ref() {
        //         self.inner
        //             .transport
        //             .send_msg(&msg, &self.inner.addr)
        //             .await?;
        //         true
        //     } else {
        //         false
        //     }
        // };

        // if retransmited {
        //     Ok(self.add_retrans_count())
        // } else {
        //     Err(crate::error::Error::Io(io::Error::new(
        //         io::ErrorKind::Other,
        //         "No message to retransmit",
        //     )))
        // }
    }

    async fn send_request(&self, msg: &OutgoingRequest) -> Result<()> {
        log::debug!(
            "<= Request {} to /{}",
            msg.message.req_line.method,
            msg.send_info.destination
        );
        let buf = msg.encoded.as_ref();

        // self.inner
        //     .transport
        //     .send_msg(&buf, &self.inner.addr)
        //     .await?;
        // self.set_last_msg(buf);
        Ok(())
    }

    async fn send_response(&self, mut msg: OutgoingResponse) -> Result<()> {
        let code = msg.message.status_line.code;
        log::debug!("=> Response {} {}", code.as_u16(), msg.message.reason());
        // self.inner.endpoint.send_outgoing_response(&mut msg).await?;
        Ok(())
    }
}

impl Drop for Inner {
    fn drop(&mut self) {

        // log::trace!(
        //     "Dropping Transaction [{}] ({:p})",
        //     self.state
        //         .lock()
        //         .unwrap()
        //         .last_status_code
        //         .unwrap()
        //         .as_u16(),
        //     self
        // )
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum Role {
    UAS,
    UAC,
}

#[derive(Default)]
/// Builder for creating a new SIP `Transaction`.
pub struct Builder {
    method: Option<SipMethod>,
    role: Option<Role>,
    endpoint: Option<Endpoint>,
    transaction_key: Option<TransactionKey>,
    transaction_state: Option<TransactionState>,
    retransmit_count: Option<u32>,
}

impl Builder {
    /// Sets the key used to identify the transaction.
    pub fn set_method(&mut self, method: SipMethod) -> &mut Self {
        self.method = Some(method);
        self
    }
    /// Sets the role of the transaction.
    pub fn set_role(&mut self, role: Role) -> &mut Self {
        self.role = Some(role);
        self
    }
    /// Sets the endpoint associated with the transaction.
    pub fn set_endpoint(&mut self, endpoint: Endpoint) -> &mut Self {
        self.endpoint = Some(endpoint);
        self
    }

    /// Sets the key used to identify the transaction.
    pub fn set_key(&mut self, key: TransactionKey) -> &mut Self {
        self.transaction_key = Some(key);
        self
    }
    /// Sets the transaction state.
    pub fn set_state(&mut self, state: TransactionState) -> &mut Self {
        self.transaction_state = Some(state);
        self
    }

    /// Set the retransmission count.
    pub fn set_retransmit_count(&mut self, retransmit_count: u32) -> &mut Self {
        self.retransmit_count = Some(retransmit_count);
        self
    }
    /// Finalize the builder into a `Transaction`.
    pub fn build(self) -> Transaction {
        let inner = Inner {
            method: self.method.expect("Method is required"),
            role: self.role.expect("Role is required"),
            endpoint: self.endpoint.expect("Endpoint is required"),
            key: self.transaction_key.expect("Key is required"),
            state: self.transaction_state.unwrap_or_default().into(),
            retransmit_count: self.retransmit_count.unwrap_or_default().into(),
        };

        let tx = Transaction {
            inner: Arc::new(inner),
        };

        log::trace!(
            "Transaction Created [{:#?}] ({:p})",
            tx.inner.role,
            tx.inner
        );

        tx
    }
}
