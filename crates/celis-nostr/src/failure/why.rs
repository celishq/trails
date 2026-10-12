use serde::{Deserialize, Serialize};

/// What became of one request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "why")]
pub enum Why {
    /// The host never answered — no route to it, DNS, TLS, or a timeout.
    Unreachable,
    /// The host answered, refusing, in the words it refused with. `after` is the seconds it said to wait
    /// before asking again, where it said any.
    Refused {
        status: u16,
        body: String,
        #[serde(default)]
        after: Option<u64>,
    },
    /// The host answered, in a shape this build does not read.
    Unreadable { detail: String },
}
