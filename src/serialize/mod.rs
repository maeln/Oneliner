use std::collections::HashMap;
use std::hash::Hash;
use std::mem;

pub mod errors;
use self::errors::{Error, Result};

pub trait Serializable {
    fn serialize(&self) -> Result<Vec<u8>>;
}

pub trait Unserializable<T: Sized> {
    fn unserialize(&[u8]) -> Result<T>;
}

impl Serializable for i32 {
    fn serialize(&self) -> Result<Vec<u8>> {
        let value = unsafe { mem::transmute::<i32, [u8; 4]>(self.to_le()).to_vec() };
        Ok(value)
    }
}

impl Unserializable<i32> for i32 {
    fn unserialize(bytes: &[u8]) -> Result<i32> {
        if bytes.len() > 4 {
            return Err(Error::new_too_much_bytes());
        } else if bytes.len() < 4 {
            return Err(Error::new_not_enough_bytes());
        }

        let value = unsafe {
            mem::transmute::<[u8; 4], i32>([bytes[0], bytes[1], bytes[2], bytes[3]]).to_le()
        };
        Ok(value)
    }
}

impl<T: Serializable + Sized> Serializable for Vec<T> {
    fn serialize(&self) -> Result<Vec<u8>> {
        let mut bytes: Vec<u8> = Vec::with_capacity(self.len() * mem::size_of::<T>());
        for i in self.iter() {
            bytes.extend(&i.serialize()?);
        }
        Ok(bytes)
    }
}
impl<T: Sized + Unserializable<T>> Unserializable<Vec<T>> for Vec<T> {
    fn unserialize(bytes: &[u8]) -> Result<Vec<T>> {
        let chunks = bytes.chunks(mem::size_of::<T>());
        let mut res: Vec<T> = Vec::with_capacity(chunks.len());
        for chunk in chunks {
            let val: T = T::unserialize(chunk)?;
            res.push(val);
        }
        Ok(res)
    }
}

impl Serializable for String {
    fn serialize(&self) -> Result<Vec<u8>> {
        let mut bytes: Vec<u8> = self.as_bytes().to_vec();
        bytes.push(b'\0');
        Ok(bytes)
    }
}

impl<K: Serializable + Sized + Eq + Hash, V: Serializable + Sized> Serializable for HashMap<K, V> {
    fn serialize(&self) -> Result<Vec<u8>> {
        let capacity = self.len() * (mem::size_of::<K>() + mem::size_of::<V>());
        let mut bytes: Vec<u8> = Vec::with_capacity(capacity);
        for (key, val) in self.iter() {
            bytes.extend(&key.serialize()?);
            bytes.extend(&val.serialize()?);
        }
        Ok(bytes)
    }
}

impl<K: Unserializable<K> + Sized + Eq + Hash, V: Unserializable<V> + Sized>
    Unserializable<HashMap<K, V>> for HashMap<K, V>
{
    fn unserialize(bytes: &[u8]) -> Result<HashMap<K, V>> {
        let key_size = mem::size_of::<K>();
        let value_size = mem::size_of::<V>();
        let map_capacity = bytes.len() / (key_size + value_size);

        let mut map: HashMap<K, V> = HashMap::with_capacity(map_capacity);

        let mut i: usize = 0;
        while i < bytes.len() {
            let key = K::unserialize(&bytes[i..i + key_size])?;
            let value = V::unserialize(&bytes[i + key_size..(i + key_size + value_size)])?;
            map.insert(key, value);
            i += key_size + value_size;
        }

        Ok(map)
    }
}
