//! Ticker symbol
use codec::{Decode, Encode};

const TICKER_LEN: usize = 12;

/// Ticker symbol identified with a name.
#[derive(Decode, Encode, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ticker(pub [u8; TICKER_LEN]);

impl Default for Ticker {
    fn default() -> Self {
        Ticker([0u8; TICKER_LEN])
    }
}

impl Ticker {
    /// Converts a byte slice to an uppercase ASCII ticker, trimming the result to 12 bytes.
    pub fn from_slice(s: &[u8]) -> Self {
        let mut ticker = [0u8; TICKER_LEN];
        for (i, b) in s
            .iter()
            .map(|b| b.to_ascii_uppercase())
            .enumerate()
            .take(TICKER_LEN)
        {
            ticker[i] = b;
        }
        Ticker(ticker)
    }

    /// Converts the ticker to canonical uppercase ASCII notation.
    pub fn canonize(&mut self) {
        for i in 0..TICKER_LEN {
            self.0[i] = self.0[i].to_ascii_uppercase();
        }
    }

    /// Computes the effective length of the ticker, that is, the length of the minimal prefix after
    /// which only zeros appear.
    pub fn len(&self) -> usize {
        for i in TICKER_LEN - 1..0 {
            if self.0[i] != 0 {
                return i + 1;
            }
        }
        0
    }
}
