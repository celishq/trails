//! A guest link as its URL carries it — the link's key and the relay its store answers on, in one bech32
//! string of NIP-19 TLV entries.

use crate::Account;
use anyhow::{anyhow, bail, Result};
use bech32::{Bech32, Hrp};

const NLINK: Hrp = Hrp::parse_unchecked("nlink");
const KEY: u8 = 0;
const RELAY: u8 = 1;

/// The key a guest link reads with, and the relay its store answers on.
pub struct Nlink {
    pub key: Account,
    pub relay: String,
}

impl Nlink {
    /// This link as the `nlink1…` string a URL carries after `#`. Refused for a relay URL over 255 bytes,
    /// which one TLV entry cannot hold.
    pub fn text(&self) -> Result<String> {
        let Ok(length) = u8::try_from(self.relay.len()) else {
            bail!("a relay URL over 255 bytes does not fit an nlink");
        };
        let mut data = vec![KEY, 32];
        data.extend(self.key.secret());
        data.extend([RELAY, length]);
        data.extend(self.relay.as_bytes());
        Ok(bech32::encode::<Bech32>(NLINK, &data)?)
    }

    /// The link `text` states. Refused unless it is an `nlink` holding one key and one relay.
    pub fn parse(text: &str) -> Result<Self> {
        let (hrp, data) = bech32::decode(text).map_err(|e| anyhow!("not bech32: {e}"))?;
        if hrp != NLINK {
            bail!("not an nlink");
        }
        let (mut key, mut relay) = (None, None);
        let mut rest = data.as_slice();
        while let [kind, length, tail @ ..] = rest {
            let length = *length as usize;
            if tail.len() < length {
                bail!("an nlink entry runs past its end");
            }
            let (value, after) = tail.split_at(length);
            match *kind {
                KEY => key = Some(Account::new(value.try_into()?)?),
                RELAY => relay = Some(String::from_utf8(value.to_vec())?),
                _ => {}
            }
            rest = after;
        }
        Ok(Self {
            key: key.ok_or_else(|| anyhow!("an nlink names no key"))?,
            relay: relay.ok_or_else(|| anyhow!("an nlink names no relay"))?,
        })
    }
}

#[cfg(test)]
mod tests;
