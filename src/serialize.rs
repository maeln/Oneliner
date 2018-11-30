use std::collections::HashMap;
use std::hash::Hash;
use std::mem;

pub trait Serializable {
    fn serialize(&self) -> Vec<u8>;
}

pub trait Unserializable {
    fn unserialize(&[u8]) -> Self;
}

impl Serializable for i32 {
    fn serialize(&self) -> Vec<u8> {
        unsafe { mem::transmute::<i32, [u8; 4]>(self.to_le()).to_vec() }
    }
}

impl Unserializable for i32 {
    fn unserialize(bytes: &[u8]) -> i32 {
        if bytes.len() != 4 {
            panic!("A i32 should be 4 bytes, instead it is {}", bytes.len());
        }
        unsafe { mem::transmute::<[u8; 4], i32>([bytes[0], bytes[1], bytes[2], bytes[3]]).to_le() }
    }
}

impl<T: Serializable + Sized> Serializable for Vec<T> {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::with_capacity(self.len() * mem::size_of::<T>());
        for i in self.iter() {
            bytes.extend(&i.serialize());
        }
        bytes
    }
}
impl<T: Sized + Unserializable> Unserializable for Vec<T> {
    fn unserialize(bytes: &[u8]) -> Vec<T> {
        bytes
            .chunks(mem::size_of::<T>())
            .map(|chunk| T::unserialize(chunk))
            .collect()
    }
}

impl Serializable for String {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = self.as_bytes().to_vec();
        bytes.push(b'\0');
        bytes
    }
}

impl<K: Serializable + Sized + Eq + Hash, V: Serializable + Sized> Serializable for HashMap<K, V> {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        for (key, val) in self.iter() {
            bytes.extend(&key.serialize());
            bytes.extend(&val.serialize());
        }
        bytes
    }
}

impl<K: Unserializable + Sized + Eq + Hash, V: Unserializable + Sized> Unserializable
    for HashMap<K, V>
{
    fn unserialize(bytes: &[u8]) -> HashMap<K, V> {
        let key_size = mem::size_of::<K>();
        let value_size = mem::size_of::<V>();
        let map_capacity = bytes.len() / (key_size + value_size);

        let mut map: HashMap<K, V> = HashMap::with_capacity(map_capacity);

        let mut i: usize = 0;
        while i < bytes.len() {
            let key = K::unserialize(&bytes[i..i + key_size]);
            let value = V::unserialize(&bytes[i + key_size..(i + key_size + value_size)]);
            map.insert(key, value);
            i += key_size + value_size;
        }

        map
    }
}
