#![forbid(unsafe_code)]

//! Consensus-adjacent mining primitives.
//!
//! Byte order is explicit:
//! - [`Hash256`] stores the 32 bytes used by Bitcoin-family wire serialization.
//! - display hex is the reversed, human-facing block-hash/txid notation.
//! - [`Target256`] is an unsigned numeric value stored as little-endian u32 limbs.

use core::fmt;
use core::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveError {
    InvalidHexLength { expected: usize, actual: usize },
    InvalidHexCharacter { index: usize },
    InvalidCompactTarget { bits: u32, negative: bool, overflow: bool },
    ZeroTarget,
}

impl fmt::Display for PrimitiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHexLength { expected, actual } => {
                write!(f, "invalid hex length: expected {expected}, got {actual}")
            }
            Self::InvalidHexCharacter { index } => write!(f, "invalid hex character at {index}"),
            Self::InvalidCompactTarget { bits, negative, overflow } => write!(
                f,
                "invalid compact target 0x{bits:08x}: negative={negative}, overflow={overflow}"
            ),
            Self::ZeroTarget => f.write_str("target must be non-zero"),
        }
    }
}

impl std::error::Error for PrimitiveError {}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Hash256([u8; 32]);

pub type BlockHash = Hash256;
pub type Txid = Hash256;

impl Hash256 {
    pub const ZERO: Self = Self([0; 32]);

    /// Construct from the 32 bytes used by Bitcoin-family uint256 wire serialization.
    pub const fn from_wire_le(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn to_wire_le_bytes(self) -> [u8; 32] {
        self.0
    }

    pub fn from_display_hex(s: &str) -> Result<Self, PrimitiveError> {
        if s.len() != 64 {
            return Err(PrimitiveError::InvalidHexLength {
                expected: 64,
                actual: s.len(),
            });
        }
        let mut display = [0_u8; 32];
        decode_hex_into(s.as_bytes(), &mut display)?;
        display.reverse();
        Ok(Self(display))
    }

    pub fn to_display_hex(self) -> String {
        let mut bytes = self.0;
        bytes.reverse();
        encode_hex(&bytes)
    }

    pub const fn as_wire_le_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Debug for Hash256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Hash256").field(&self.to_display_hex()).finish()
    }
}

impl fmt::Display for Hash256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_display_hex())
    }
}

impl FromStr for Hash256 {
    type Err = PrimitiveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_display_hex(s)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct CompactTarget(pub u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Target256 {
    limbs: [u32; 8],
}

impl fmt::Debug for Target256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Target256(0x{})", encode_hex(&self.to_be_bytes()))
    }
}

impl Ord for Target256 {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        for i in (0..8).rev() {
            match self.limbs[i].cmp(&other.limbs[i]) {
                core::cmp::Ordering::Equal => {}
                non_eq => return non_eq,
            }
        }
        core::cmp::Ordering::Equal
    }
}

impl PartialOrd for Target256 {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Target256 {
    pub const ZERO: Self = Self { limbs: [0; 8] };

    pub const fn from_le_limbs(limbs: [u32; 8]) -> Self {
        Self { limbs }
    }

    pub fn from_be_bytes(bytes: [u8; 32]) -> Self {
        let mut limbs = [0_u32; 8];
        for (idx, chunk) in bytes.rchunks_exact(4).enumerate() {
            limbs[idx] = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }
        Self { limbs }
    }

    pub fn to_be_bytes(self) -> [u8; 32] {
        let mut out = [0_u8; 32];
        for (idx, limb) in self.limbs.iter().enumerate() {
            let start = 32 - (idx + 1) * 4;
            out[start..start + 4].copy_from_slice(&limb.to_be_bytes());
        }
        out
    }

    pub fn to_wire_le_bytes(self) -> [u8; 32] {
        let mut out = self.to_be_bytes();
        out.reverse();
        out
    }

    pub const fn is_zero(self) -> bool {
        let l = self.limbs;
        l[0] == 0 && l[1] == 0 && l[2] == 0 && l[3] == 0 &&
        l[4] == 0 && l[5] == 0 && l[6] == 0 && l[7] == 0
    }

    pub fn from_compact(bits: CompactTarget) -> Result<Self, PrimitiveError> {
        let compact = bits.0;
        let size = compact >> 24;
        let mut word = compact & 0x007f_ffff;
        let negative = word != 0 && (compact & 0x0080_0000) != 0;
        let overflow = word != 0
            && (size > 34
                || (word > 0xff && size > 33)
                || (word > 0xffff && size > 32));

        if negative || overflow {
            return Err(PrimitiveError::InvalidCompactTarget {
                bits: compact,
                negative,
                overflow,
            });
        }

        let value = if size <= 3 {
            word >>= 8 * (3 - size);
            Self::from_u32(word)
        } else {
            Self::from_u32(word).shl_bytes((size - 3) as usize)
        };
        Ok(value)
    }

    pub fn to_compact(self) -> CompactTarget {
        if self.is_zero() {
            return CompactTarget(0);
        }

        let mut size = self.bit_length().div_ceil(8) as u32;
        let mut compact = if size <= 3 {
            self.low_u64() << (8 * (3 - size))
        } else {
            self.shr_bytes((size - 3) as usize).low_u64()
        } as u32;

        if (compact & 0x0080_0000) != 0 {
            compact >>= 8;
            size += 1;
        }

        compact |= size << 24;
        CompactTarget(compact)
    }

    pub const fn from_u32(v: u32) -> Self {
        Self {
            limbs: [v, 0, 0, 0, 0, 0, 0, 0],
        }
    }

    pub fn mul_u32(self, rhs: u32) -> Self {
        let mut out = [0_u32; 8];
        let mut carry = 0_u64;
        for (idx, limb) in self.limbs.iter().enumerate() {
            let product = (*limb as u64) * (rhs as u64) + carry;
            out[idx] = product as u32;
            carry = product >> 32;
        }
        Self { limbs: out }
    }

    pub fn div_u32(self, rhs: u32) -> Self {
        assert!(rhs != 0, "division by zero");
        let mut out = [0_u32; 8];
        let mut rem = 0_u64;
        for idx in (0..8).rev() {
            let n = (rem << 32) | self.limbs[idx] as u64;
            out[idx] = (n / rhs as u64) as u32;
            rem = n % rhs as u64;
        }
        Self { limbs: out }
    }

    pub fn meets(&self, hash: Hash256) -> bool {
        let bytes = hash.to_wire_le_bytes();
        let mut limbs = [0_u32; 8];
        for (idx, chunk) in bytes.chunks_exact(4).enumerate() {
            limbs[idx] = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }
        Self { limbs } <= *self
    }

    fn bit_length(self) -> usize {
        for idx in (0..8).rev() {
            let limb = self.limbs[idx];
            if limb != 0 {
                return idx * 32 + (32 - limb.leading_zeros() as usize);
            }
        }
        0
    }

    fn low_u64(self) -> u64 {
        self.limbs[0] as u64 | ((self.limbs[1] as u64) << 32)
    }

    fn shl_bytes(self, bytes: usize) -> Self {
        if bytes >= 32 {
            return Self::ZERO;
        }
        let bit_shift = bytes * 8;
        let word_shift = bit_shift / 32;
        let rem = bit_shift % 32;
        let mut out = [0_u32; 8];
        for src in 0..8 {
            let dst = src + word_shift;
            if dst >= 8 {
                continue;
            }
            out[dst] |= self.limbs[src] << rem;
            if rem != 0 && dst + 1 < 8 {
                out[dst + 1] |= self.limbs[src] >> (32 - rem);
            }
        }
        Self { limbs: out }
    }

    fn shr_bytes(self, bytes: usize) -> Self {
        if bytes >= 32 {
            return Self::ZERO;
        }
        let bit_shift = bytes * 8;
        let word_shift = bit_shift / 32;
        let rem = bit_shift % 32;
        let mut out = [0_u32; 8];
        for dst in 0..8 {
            let src = dst + word_shift;
            if src >= 8 {
                break;
            }
            out[dst] |= self.limbs[src] >> rem;
            if rem != 0 && src + 1 < 8 {
                out[dst] |= self.limbs[src + 1] << (32 - rem);
            }
        }
        Self { limbs: out }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Satoshis(pub i64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BitcoinAmount(pub i64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HashesPerSecond(pub u128);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Terahashes(pub u128);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Petahashes(pub u128);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SatsPerPhDay(pub i128);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Difficulty {
    pub difficulty_one_target: Target256,
    pub current_target: Target256,
}

impl Difficulty {
    pub fn new(
        difficulty_one_target: Target256,
        current_target: Target256,
    ) -> Result<Self, PrimitiveError> {
        if current_target.is_zero() {
            return Err(PrimitiveError::ZeroTarget);
        }
        Ok(Self { difficulty_one_target, current_target })
    }
}

pub fn compact_to_target(bits: CompactTarget) -> Result<Target256, PrimitiveError> {
    Target256::from_compact(bits)
}

pub fn target_to_compact(target: Target256) -> CompactTarget {
    target.to_compact()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BitcoinBlockHeader {
    pub version: i32,
    pub previous_block: BlockHash,
    pub merkle_root: Hash256,
    pub time: u32,
    pub bits: CompactTarget,
    pub nonce: u32,
}

impl BitcoinBlockHeader {
    pub fn serialize_wire(self) -> [u8; 80] {
        let mut out = [0_u8; 80];
        out[0..4].copy_from_slice(&self.version.to_le_bytes());
        out[4..36].copy_from_slice(self.previous_block.as_wire_le_bytes());
        out[36..68].copy_from_slice(self.merkle_root.as_wire_le_bytes());
        out[68..72].copy_from_slice(&self.time.to_le_bytes());
        out[72..76].copy_from_slice(&self.bits.0.to_le_bytes());
        out[76..80].copy_from_slice(&self.nonce.to_le_bytes());
        out
    }

    pub fn hash(self) -> BlockHash {
        sha256d(&self.serialize_wire())
    }
}

pub fn sha256(data: &[u8]) -> [u8; 32] {
    const H0: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    const K: [u32; 64] = [
        0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
        0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
        0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
        0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
        0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
        0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
        0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
        0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2,
    ];

    let mut h = H0;
    let bit_len = (data.len() as u64) * 8;
    let mut padded = data.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.chunks_exact(64) {
        let mut w = [0_u32; 64];
        for (i, word) in chunk.chunks_exact(4).enumerate().take(16) {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut out = [0_u8; 32];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

pub fn sha256d(data: &[u8]) -> Hash256 {
    Hash256::from_wire_le(sha256(&sha256(data)))
}

pub fn bitcoin_merkle_root(txids: &[Txid]) -> Option<Hash256> {
    if txids.is_empty() {
        return None;
    }
    let mut level = txids.to_vec();
    while level.len() > 1 {
        if level.len() % 2 == 1 {
            let last = *level.last().expect("non-empty level");
            level.push(last);
        }
        let mut next = Vec::with_capacity(level.len() / 2);
        for pair in level.chunks_exact(2) {
            let mut bytes = [0_u8; 64];
            bytes[..32].copy_from_slice(pair[0].as_wire_le_bytes());
            bytes[32..].copy_from_slice(pair[1].as_wire_le_bytes());
            next.push(sha256d(&bytes));
        }
        level = next;
    }
    level.into_iter().next()
}

fn decode_hex_into(input: &[u8], output: &mut [u8]) -> Result<(), PrimitiveError> {
    debug_assert_eq!(input.len(), output.len() * 2);
    for (idx, byte) in output.iter_mut().enumerate() {
        let hi = hex_nibble(input[idx * 2]).ok_or(PrimitiveError::InvalidHexCharacter { index: idx * 2 })?;
        let lo = hex_nibble(input[idx * 2 + 1]).ok_or(PrimitiveError::InvalidHexCharacter { index: idx * 2 + 1 })?;
        *byte = (hi << 4) | lo;
    }
    Ok(())
}

fn hex_nibble(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_known_vector() {
        assert_eq!(
            encode_hex(&sha256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn sha256d_empty_vector() {
        assert_eq!(
            sha256d(b"").to_display_hex(),
            "56944c5d3f98413ef45cf54545538103cc9f298e0575820ad3591376e2e0f65d"
        );
    }

    #[test]
    fn display_hex_round_trip_reverses_wire_bytes() {
        let hash = Hash256::from_display_hex(
            "0000000000000000000b4d0c4c6f62a5f6d02e52d9f8f629c25c54f9a3f68f45"
        ).unwrap();
        let mut expected = [
            0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x00,0x0b,0x4d,0x0c,0x4c,0x6f,0x62,0xa5,
            0xf6,0xd0,0x2e,0x52,0xd9,0xf8,0xf6,0x29,0xc2,0x5c,0x54,0xf9,0xa3,0xf6,0x8f,0x45,
        ];
        expected.reverse();
        assert_eq!(hash.to_wire_le_bytes(), expected);
    }

    #[test]
    fn compact_target_bitcoin_difficulty_one() {
        let bits = CompactTarget(0x1d00ffff);
        let target = Target256::from_compact(bits).unwrap();
        assert_eq!(target.to_compact(), bits);
        assert_eq!(
            encode_hex(&target.to_be_bytes()),
            "00000000ffff0000000000000000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn compact_target_round_trip_examples() {
        for raw in [0x1d00ffff, 0x1b0404cb, 0x190d1fce, 0x1e0ffff0] {
            let compact = CompactTarget(raw);
            let target = Target256::from_compact(compact).unwrap();
            assert_eq!(target.to_compact(), compact);
        }
    }

    #[test]
    fn compact_target_rejects_negative_and_overflow() {
        assert!(matches!(
            Target256::from_compact(CompactTarget(0x1d80ffff)),
            Err(PrimitiveError::InvalidCompactTarget { negative: true, .. })
        ));
        assert!(matches!(
            Target256::from_compact(CompactTarget(0x23010000)),
            Err(PrimitiveError::InvalidCompactTarget { overflow: true, .. })
        ));
    }

    #[test]
    fn small_integer_mul_div_matches_ordered_core_operations() {
        let t = Target256::from_compact(CompactTarget(0x190d1fce)).unwrap();
        let adjusted = t.mul_u32(690).div_u32(750);
        assert!(adjusted < t);
        assert_eq!(adjusted.to_compact(), CompactTarget(0x190c1305));
    }

    #[test]
    fn bitcoin_genesis_header_serialization_and_hash() {
        let header = BitcoinBlockHeader {
            version: 1,
            previous_block: Hash256::ZERO,
            merkle_root: Hash256::from_display_hex(
                "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b"
            ).unwrap(),
            time: 1_231_006_505,
            bits: CompactTarget(0x1d00ffff),
            nonce: 2_083_236_893,
        };
        assert_eq!(header.serialize_wire().len(), 80);
        assert_eq!(
            header.hash().to_display_hex(),
            "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"
        );
    }

    #[test]
    fn merkle_duplicates_odd_leaf() {
        let a = sha256d(b"a");
        let b = sha256d(b"b");
        let c = sha256d(b"c");
        let root = bitcoin_merkle_root(&[a, b, c]).unwrap();

        let ab = {
            let mut bytes = [0_u8; 64];
            bytes[..32].copy_from_slice(a.as_wire_le_bytes());
            bytes[32..].copy_from_slice(b.as_wire_le_bytes());
            sha256d(&bytes)
        };
        let cc = {
            let mut bytes = [0_u8; 64];
            bytes[..32].copy_from_slice(c.as_wire_le_bytes());
            bytes[32..].copy_from_slice(c.as_wire_le_bytes());
            sha256d(&bytes)
        };
        let expected = {
            let mut bytes = [0_u8; 64];
            bytes[..32].copy_from_slice(ab.as_wire_le_bytes());
            bytes[32..].copy_from_slice(cc.as_wire_le_bytes());
            sha256d(&bytes)
        };
        assert_eq!(root, expected);
    }
}
