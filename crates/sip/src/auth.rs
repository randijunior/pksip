pub(crate) mod challenge;
pub(crate) mod credential;

pub(crate) const CNONCE: &str = "cnonce";
pub(crate) const QOP: &str = "qop";
pub(crate) const NC: &str = "nc";
pub(crate) const NEXTNONCE: &str = "nextnonce";
pub(crate) const RSPAUTH: &str = "rspauth";

const DIGEST: &str = "Digest";
const REALM: &str = "realm";
const USERNAME: &str = "username";
const NONCE: &str = "nonce";
const URI: &str = "uri";
const RESPONSE: &str = "response";
const ALGORITHM: &str = "algorithm";
const OPAQUE: &str = "opaque";
const DOMAIN: &str = "domain";
const STALE: &str = "stale";
