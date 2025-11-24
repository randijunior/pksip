pub mod invite;
pub mod non_invite;

use invite::ClientInviteTx;
use non_invite::ClientNonInviteTx;

#[derive(Clone)]
/// An Client Transaction, either Invite or NonInvite.
pub enum ClientTx {
    /// An NonInvite Client Transaction.
    NonInvite(ClientNonInviteTx),
    /// An Invite Client Transaction.
    Invite(ClientInviteTx),
}
