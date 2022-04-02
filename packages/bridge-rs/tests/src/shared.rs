use casper_types::U256;

use std::fmt::Write;

// utils

use sha2::{Digest, Sha256};

pub fn sha256(data: &[u8]) -> Vec<u8> {
    let mut instance = Sha256::new();
    instance.update(data);
    let result = instance.finalize();
    result.to_vec()
}

pub fn merge_bytes(vecs: Vec<Vec<u8>>) -> Vec<u8> {
    let mut data = Vec::new();

    for vec in vecs {
        data.extend(vec);
    }

    data
}

// pub fn decode_hex(s: &str) -> Result<Vec<u8>> {
//   (0..s.len())
//     .step_by(2)
//     .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
//     .collect()
// }

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        write!(&mut s, "{:02x}", b).unwrap();
    }
    s
}

// compatible with abi encode (solidity)
pub fn u256_to_bytes(u: &U256) -> Vec<u8> {
    let mut buffer = [0u8; 32];
    u.to_big_endian(&mut buffer);
    buffer.to_vec()
}

pub fn u256_to_hex(u: &U256) -> String {
    let bytes = u256_to_bytes(u);
    encode_hex(&bytes)
}

pub fn pad_with_8_bytes(bytes: Vec<u8>) -> Vec<u8> {
    let pad_bytes = vec![0; 8];

    merge_bytes(vec![pad_bytes, bytes])
}
