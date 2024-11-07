pub(crate) mod challenge;
pub(crate) mod credential;

const DIGEST: &str = "Digest";
const REALM: &str = "realm";
const USERNAME: &str = "username";
const NONCE: &str = "nonce";
const URI: &str = "uri";
const RESPONSE: &str = "response";
const ALGORITHM: &str = "algorithm";
pub(crate) const CNONCE: &str = "cnonce";
const OPAQUE: &str = "opaque";
pub(crate) const QOP: &str = "qop";
pub(crate) const NC: &str = "nc";
const DOMAIN: &str = "domain";
const STALE: &str = "stale";
pub(crate) const NEXTNONCE: &str = "nextnonce";
pub(crate) const RSPAUTH: &str = "rspauth";
