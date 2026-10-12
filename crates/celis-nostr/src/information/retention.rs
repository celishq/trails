use serde::Deserialize;
use serde_json::Value;

/// One retention rule: how long the relay keeps the kinds it names. A rule may name a kind on its own or a
/// `[first, last]` range of them.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Retention {
    #[serde(default)]
    pub(super) kinds: Vec<Value>,
}
