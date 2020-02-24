//! Ticker symbol
use codec::{Decode, Encode, Error, Input};
use sp_std::cmp::min;

const TICKER_LEN: usize = 12;

/// Ticker symbol.
///
/// This type stores fixed-length case-sensitive byte strings. Any value of this type that is
/// received by a Substrate module call method has to be converted to canonical uppercase
/// representation using [`Ticker::canonize`].
#[derive(Encode, Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ticker([u8; TICKER_LEN]);

impl Default for Ticker {
    fn default() -> Self {
        Ticker([0u8; TICKER_LEN])
    }
}

impl From<&[u8]> for Ticker {
    fn from(s: &[u8]) -> Self {
        let max_len = min(TICKER_LEN, s.len());
        let mut ticker = [0u8; TICKER_LEN];

        // Copy and force to upper.
        ticker[..max_len].copy_from_slice(&s[..max_len]);
        ticker.make_ascii_uppercase();

        Ticker(ticker)
    }
}

/// It custom decoder enforces to upper case.
impl Decode for Ticker {
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
        let ticker = <[u8; TICKER_LEN]>::decode(input)?;

        Ok(Self::from(&ticker[..]))
    }
}

impl Ticker {
    /// Computes the effective length of the ticker, that is, the length of the minimal prefix after
    /// which only zeros appear.
    pub fn len(&self) -> usize {
        for i in (0..TICKER_LEN).rev() {
            if self.0[i] != 0 {
                return i + 1;
            }
        }
        0
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticker_test() {
        // 0. Simple
        let s1 = b"abcdabcdabcd";
        let t1 = Ticker::from(&s1[..]);
        assert_eq!(t1.len(), 12);
        assert_eq!(t1.as_slice(), b"ABCDABCDABCD");

        // 1. More characters than expected.
        let s2 = b"abcdabcdabcdabcd";
        let t2 = Ticker::from(&s2[..]);
        assert_eq!(t2.len(), 12);
        assert_eq!(t2.as_slice(), b"ABCDABCDABCD");

        // 2. Less characters than expected.
        let s3 = b"abcd";
        let t3 = Ticker::from(&s3[..]);
        assert_eq!(t3.len(), 4);
        assert_eq!(t3.as_slice(), b"ABCD\0\0\0\0\0\0\0\0");
    }

    #[test]
    fn parity_scale_codec() {
        let s = b"abcd";
        let t = Ticker::from(&s[..]);

        let t_encoded = t.encode();
        let t2 = Ticker::decode(&mut &t_encoded[..]).unwrap();

        assert_eq!(t, t2);
    }
}
