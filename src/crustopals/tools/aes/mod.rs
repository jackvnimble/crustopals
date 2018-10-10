pub mod byte_operations;

use self::byte_operations::s_box;
use crustopals::tools;

#[derive(Debug)]
pub struct Word {
  bytes: [u8; 4],
}

impl Word {
  pub fn new(slice: &[u8]) -> Word {
    let mut bytes: [u8; 4] = Default::default();
    bytes.copy_from_slice(slice);
    Word { bytes }
  }

  pub fn xor(&self, other: &Word) -> Word {
    Word::new(&tools::xor_bytes(&self.bytes, &other.bytes)[..])
  }

  pub fn rotated(&self) -> Word {
    Word::new(&[self.bytes[1], self.bytes[2], self.bytes[3], self.bytes[0]])
  }

  pub fn sbox_mapped(&self) -> Word {
    Word::new(&[
      s_box(self.bytes[0]),
      s_box(self.bytes[1]),
      s_box(self.bytes[2]),
      s_box(self.bytes[3]),
    ])
  }
}

impl PartialEq for Word {
  fn eq(&self, other: &Word) -> bool {
    self.bytes == other.bytes
  }
}

pub fn encrypt_message(bytes: &[u8], key: &[u8]) -> Vec<u8> {
  let round_keys = key_schedule(key);
  let padded_bytes = pad_bytes(bytes);
  for block in padded_bytes.chunks(16) {
    println!("the block looks like {:?}", block);
  }
  vec![b'0']
}

pub fn key_schedule(key: &[u8]) -> Vec<Word> {
  // takes 4 words (32 bits each) and transform them into 44 words
  if key.len() != 16 {
    panic!("Wrong size key. Must be 16 bytes.");
  }
  let mut expanded_key: Vec<Word> = vec![];

  for word in key.chunks(4) {
    expanded_key.push(Word::new(&word))
  }

  for word_idx in 4..44 {
    let word: Word;
    {
      let one_ago = &expanded_key[word_idx - 1];
      let four_ago = &expanded_key[word_idx - 4];
      if word_idx % 4 == 0 {
        let rconi = rcon(word_idx);
        let rot_and_sboxed = one_ago.rotated().sbox_mapped();
        word = four_ago.xor(&rot_and_sboxed).xor(&rconi);
      } else {
        word = one_ago.xor(four_ago);
      }
    }
    expanded_key.push(word);
  }

  expanded_key
}

fn pad_bytes(bytes: &[u8]) -> Vec<u8> {
  let mut byte_vec = bytes.to_vec();
  let num_bytes = 16 - (byte_vec.len() % 16);
  byte_vec.extend(padding_bytes(num_bytes));
  byte_vec
}

fn padding_bytes(num_bytes: usize) -> Vec<u8> {
  let mut padding: Vec<u8> = vec![];
  for i in 0..num_bytes {
    padding.push(num_bytes as u8);
  }
  padding
}

fn rcon(word_idx: usize) -> Word {
  Word::new(&[rc(word_idx / 4), 0 as u8, 0 as u8, 0 as u8])
}

fn rc(idx: usize) -> u8 {
  [1, 2, 4, 8, 16, 32, 64, 128, 27, 54][idx - 1] as u8
}

#[cfg(test)]
mod tests {
  use super::*;
  extern crate hex;

  #[test]
  #[should_panic(expected = "Wrong size key. Must be 16 bytes.")]
  fn panics_with_wrong_keysize() {
    let key = b"Hello world";
    key_schedule(key);
  }

  #[test]
  fn expands_an_aes_key_into_round_keys() {
    let key = hex::decode("2b7e151628aed2a6abf7158809cf4f3c").unwrap();
    let expanded_round_key_words = [
      "2b7e1516", "28aed2a6", "abf71588", "09cf4f3c", "a0fafe17", "88542cb1",
      "23a33939", "2a6c7605", "f2c295f2", "7a96b943", "5935807a", "7359f67f",
      "3d80477d", "4716fe3e", "1e237e44", "6d7a883b", "ef44a541", "a8525b7f",
      "b671253b", "db0bad00", "d4d1c6f8", "7c839d87", "caf2b8bc", "11f915bc",
      "6d88a37a", "110b3efd", "dbf98641", "ca0093fd", "4e54f70e", "5f5fc9f3",
      "84a64fb2", "4ea6dc4f", "ead27321", "b58dbad2", "312bf560", "7f8d292f",
      "ac7766f3", "19fadc21", "28d12941", "575c006e", "d014f9a8", "c9ee2589",
      "e13f0cc8", "b6630ca6",
    ];

    let computed_round_keys = key_schedule(&key);

    for (i, word) in expanded_round_key_words.iter().enumerate() {
      assert_eq!(
        computed_round_keys[i],
        Word::new(&hex::decode(word).unwrap())
      );
    }
  }

  #[test]
  fn encrypts_messages() {
    let key = "YELLOW SUBMARINE";
    let message = String::from("here is the mess");
    let _aes_128_bit_encrypted =
      encrypt_message(message.as_bytes(), key.as_bytes());

    assert_eq!(2, 3);
  }
}