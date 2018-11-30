use std::collections::HashMap;
use std::mem;

pub trait Serializable {
    fn serialize(&self) -> Vec<u8>;
    fn unserialize(&[u8]) -> Self;
}

impl Serializable for i32 {
    fn serialize(&self) -> Vec<u8> {
        unsafe { mem::transmute::<i32, [u8; 4]>(self.to_le()).to_vec() }
    }

    fn unserialize(bytes: &[u8]) -> i32 {
        if bytes.len() != 4 {
            panic!("A i32 should be 4 bytes, instead it is {}", bytes.len());
        }
        unsafe { mem::transmute::<[u8; 4], i32>([bytes[0], bytes[1], bytes[2], bytes[3]]).to_le() }
    }
}

pub fn i32tolebytes(num: i32) -> [u8; 4] {
    unsafe { mem::transmute(num.to_le()) }
}

pub fn lebytestoi32(num: [u8; 4]) -> i32 {
    unsafe { mem::transmute::<[u8; 4], i32>(num).to_le() }
}

pub fn string_to_bytes(string: &str) -> Vec<u8> {
    let copy = string.to_string();
    let mut bytes = copy.into_bytes();
    bytes.push(b'\0');
    bytes
}

pub fn string_list_to_bytes(strings: &[String]) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    for string in strings.iter() {
        bytes.extend(string_to_bytes(string));
    }
    bytes
}

pub fn i32_list_to_bytes(integers: &[i32]) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::with_capacity(integers.len() * 4);
    for i in integers.iter() {
        bytes.extend(&i32tolebytes(*i));
    }
    bytes
}

pub fn i32_hash_to_bytes(hash: &HashMap<i32, i32>) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    for (key, val) in hash.iter() {
        bytes.extend(&i32tolebytes(*key));
        bytes.extend(&i32tolebytes(*val));
    }
    bytes
}
