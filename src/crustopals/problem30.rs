extern crate fake_simd as simd;
extern crate md4;

use self::md4::{Digest, Md4, Md4State};
use self::simd::u32x4;

pub fn generate_md4_padding(bytes: &[u8]) -> Vec<u8> {
  let num_bytes = bytes.len();
  let extra_bytes = num_bytes % 64;
  let mut last_blocks = [0u8; 128];
  last_blocks[0..extra_bytes]
    .clone_from_slice(&bytes[(num_bytes - extra_bytes)..]);
  last_blocks[extra_bytes] = 0x80u8;
  let num_bits: u64 = num_bytes as u64 * 8;
  let extra = [
    num_bits as u8,
    (num_bits >> 8) as u8,
    (num_bits >> 16) as u8,
    (num_bits >> 24) as u8,
    (num_bits >> 32) as u8,
    (num_bits >> 40) as u8,
    (num_bits >> 48) as u8,
    (num_bits >> 56) as u8,
  ];
  if extra_bytes < 56 {
    last_blocks[56..64].clone_from_slice(&extra);
    last_blocks[..64].to_vec()
  } else {
    last_blocks[120..128].clone_from_slice(&extra);
    last_blocks.to_vec()
  }
}

pub fn forge_md4_mac(
  mac: &[u8],
  msg: &[u8],
  forged_bytes: &[u8],
) -> (Vec<u8>, Vec<u8>) {
  let fake_secret = [0u8; 16]; // "guess" of 16 bytes
  let mut full_msg: Vec<u8> = vec![];
  full_msg.extend(fake_secret.to_vec());
  full_msg.extend(msg.to_vec());
  let padding = generate_md4_padding(&full_msg);

  let blks_len = (full_msg.len() as u64 / 64) * 64;
  let padding_len = padding.len() as u64;

  let mut forged_msg: Vec<u8> = vec![];
  forged_msg.extend(&full_msg[16..blks_len as usize].to_vec());
  forged_msg.extend(padding);

  let md4_state = md4_state_from_digest(mac);
  let length = blks_len + padding_len;
  let mut md4 = Md4::new();
  md4.length_bytes = length;
  md4.state = md4_state;
  md4.input(&forged_bytes);
  let forged_hash = md4.result().to_vec();

  forged_msg.extend(forged_bytes.to_vec());
  (forged_hash, forged_msg)
}

fn md4_state_from_digest(digest_bytes: &[u8]) -> Md4State {
  let mut state = [0u32; 4];
  for i in 0..4 {
    let offset = i * 4;
    state[i] = (digest_bytes[offset + 3] as u32) << 24
      | ((digest_bytes[offset + 2] as u32) << 16)
      | ((digest_bytes[offset + 1] as u32) << 8)
      | (digest_bytes[offset] as u32);
  }
  Md4State {
    s: u32x4(state[0], state[1], state[2], state[3]),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crustopals::query_string;
  use crustopals::tools::*;

  #[test]
  fn it_generates_padding_matching_md4() {
    let quote1 = "Thunder rolled. It rolled a 6";
    let quote2 = "Real stupidity beats artificial intelligence every time.";
    let padding1: Vec<u8> = vec![
      84, 104, 117, 110, 100, 101, 114, 32, 114, 111, 108, 108, 101, 100, 46,
      32, 73, 116, 32, 114, 111, 108, 108, 101, 100, 32, 97, 32, 54, 128, 0, 0,
      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
      232, 0, 0, 0, 0, 0, 0, 0,
    ];
    let padding2: Vec<u8> = vec![
      82, 101, 97, 108, 32, 115, 116, 117, 112, 105, 100, 105, 116, 121, 32,
      98, 101, 97, 116, 115, 32, 97, 114, 116, 105, 102, 105, 99, 105, 97, 108,
      32, 105, 110, 116, 101, 108, 108, 105, 103, 101, 110, 99, 101, 32, 101,
      118, 101, 114, 121, 32, 116, 105, 109, 101, 46, 128, 0, 0, 0, 0, 0, 0, 0,
      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
      0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
      0, 0, 0, 0, 0, 0, 0, 0, 192, 1, 0, 0, 0, 0, 0, 0,
    ];

    assert_eq!(generate_md4_padding(quote1.as_bytes()), padding1);
    assert_eq!(generate_md4_padding(quote2.as_bytes()), padding2);
  }

  #[test]
  fn it_forges_valid_md4_macs() {
    let secret_key: Vec<u8> = aes::generate_key(); //random 16 byte key
    let msg_bytes = "comment1=cooking%20MCs;userdata=foo;comment2=%20like%20a%20pound%20of%20bacon".as_bytes();
    let legit_mac = authentication::md4_mac(&secret_key, &msg_bytes);
    let desired_append_bytes = ";admin=true;".as_bytes();

    let (forged_mac, forged_msg) =
      forge_md4_mac(&legit_mac, &msg_bytes, &desired_append_bytes);

    assert!(query_string::has_admin_rights(&forged_msg));
    assert!(authentication::valid_md4_mac(
      &secret_key,
      &forged_msg,
      forged_mac
    ));
  }
}
