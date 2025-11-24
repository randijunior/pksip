use std::{collections::HashMap, sync::Mutex, time::Duration};

use crate::{
    transaction::{
        key::TransactionKey as Key, sip_transaction::Role, ClientNonInviteTx, ClientTx, ClientInviteTx, ServerInviteTx, ServerNonInviteTx, ServerTx
    }, transport::{IncomingRequest, IncomingResponse}, Result
};

type Transactions<T> = Mutex<HashMap<Key, T>>;

/// This type holds all server and client TransactionLayer
/// created by the TU (Transaction User).
#[derive(Default)]
pub struct TransactionLayer {
    client_transactions: Transactions<ClientTx>,
    server_transactions: Transactions<ServerTx>,
}

impl TransactionLayer {
    /// Remove an server transaction in the collection.
    #[inline]
    pub fn remove_server_tsx(&self, key: &Key) -> Option<ServerTx> {
        let mut map = self.server_transactions.lock().expect("Lock failed");
        map.remove(key)
    }

    /// Remove an client transaction in the collection.
    #[inline]
    pub fn remove_client_tsx(&self, key: &Key) -> Option<ClientTx> {
        let mut map = self.client_transactions.lock().expect("Lock failed");
        map.remove(key)
    }

    #[inline]
    pub(crate) fn add_server_tsx_to_map(&self, tsx: ServerNonInviteTx) {
        todo!()
        // let key = tsx.inner.key.clone();
        // let mut map = self.server_transactions.lock().expect("Lock failed");

        // map.insert(key, ServerTx::NonInvite(tsx));
    }

    #[inline]
    pub(crate) fn add_client_tsx_to_map(&self, tsx: ClientNonInviteTx) {
        let key = tsx.key().clone();
        let mut map = self.client_transactions.lock().expect("Lock failed");

        map.insert(key, ClientTx::NonInvite(tsx));
    }

    #[inline]
    pub(crate) fn add_client_inv(&self, client_inv: ClientInviteTx) {
        let key = client_inv.key().clone();
        let mut map = self.client_transactions.lock().expect("Lock failed");

        map.insert(key, ClientTx::Invite(client_inv));
    }

    #[inline]
    pub(crate) fn add_server_inv_to_map(&self, tsx: ServerInviteTx) {
        todo!()
        // let key = tsx.inner.key.clone();
        // let mut map = self.server_transactions.lock().expect("Lock failed");

        // map.insert(key, ServerTx::Invite(tsx));
    }

    fn find_server_tsx(&self, key: &Key) -> Option<ServerTx> {
        self.server_transactions
            .lock()
            .expect("Lock failed")
            .get(key)
            .cloned()
    }

    fn find_client_tsx(&self, key: &Key) -> Option<ClientTx> {
        self.client_transactions
            .lock()
            .expect("Lock failed")
            .get(key)
            .cloned()
    }

    pub(crate) async fn handle_response(&self, response: &IncomingResponse) -> Result<bool> {
        let cseq_method = response.info.mandatory_headers.cseq.method;
        let via_branch = response.info.mandatory_headers.via.branch.clone().unwrap();

        let key = Key::new_key_3261(Role::UAC, cseq_method, via_branch);
        let client_tsx = {
            match self.find_client_tsx(&key) {
                Some(tsx) => tsx,
                None => return Ok(false),
            }
        };
        let handled = match client_tsx {
            ClientTx::NonInvite(tsx) => tsx.receive(response).await?,
            ClientTx::Invite(tsx_inv) => tsx_inv.receive(response).await?,
        };

        Ok(handled)
    }

    pub(crate) async fn on_request(&self, request: &IncomingRequest) -> Result<bool> {
        let server_tsx = {
            let key = Key::from_incoming(&request.info);

            match self.find_server_tsx(&key) {
                Some(tsx) => tsx,
                None => return Ok(false),
            }
        };

        // server_tsx.receive_request(request).await?;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{endpoint, message::SipMethod};

    #[tokio::test]
    async fn test_non_invite_server_tsx() {
        /*
        let mut req = mock::request(SipMethod::Register);

        let endpoint = endpoint::EndpointBuilder::new()
            .add_transaction(TransactionLayer::default())
            .build();

        let tsx = endpoint.new_server_transaction(&mut req);

        let transactions = endpoint.transactions();
        let key = tsx.key();
        let tsx = transactions.find_server_tsx(&key);

        assert!(matches!(tsx.as_ref(), Some(ServerTx::NonInvite(_))));
        let tsx = match tsx.unwrap() {
            ServerTx::NonInvite(tsx) => tsx,
            _ => unreachable!(),
        };

        tsx.on_terminated();
        let tsx = transactions.find_server_tsx(&key);

        assert!(tsx.is_none());
         */
    }

    #[tokio::test]
    async fn test_invite_server_tsx() {
        /*
        let mut req = mock::request(SipMethod::Invite);

        let endpoint = endpoint::EndpointBuilder::new()
            .add_transaction(TransactionLayer::default())
            .build();

        let tsx = endpoint.new_inv_server_transaction(&mut req);

        let transactions = endpoint.transactions();
        let key = tsx.key();

        let tsx = transactions.find_server_tsx(&key);

        assert!(matches!(tsx.as_ref(), Some(ServerTx::Invite(_))));

        let tsx = match tsx.unwrap() {
            ServerTx::Invite(tsx) => tsx,
            _ => unreachable!(),
        };

        tsx.on_terminated();

        let tsx = transactions.find_server_tsx(&key);

        assert!(tsx.is_none());
        */
    }
}
