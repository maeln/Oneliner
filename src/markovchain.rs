use rand::prelude::*;

use std::collections::HashMap;

use regex::Regex;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use serialize::errors::{Error, Result};
use serialize::{Serializable, Unserializable};

#[derive(Default)]
pub struct MarkovChain {
    pub tokens: Vec<String>,
    pub props: Vec<HashMap<i32, i32>>,
    pub start: Vec<i32>,
    pub end: Vec<i32>,
}

impl MarkovChain {
    pub fn new() -> MarkovChain {
        MarkovChain {
            tokens: Vec::new(),
            start: Vec::new(),
            end: Vec::new(),
            props: Vec::new(),
        }
    }

    fn pick_next(&self, token: usize) -> Option<usize> {
        let prob = &self.props[token];
        if prob.len() == 0 {
            return None;
        }

        let mut rng = rand::thread_rng();
        let mut probvec: Vec<(i32, i32)> = Vec::new();
        for (k, v) in prob.iter() {
            probvec.push((k.clone(), v.clone()));
        }

        Some(probvec.choose_weighted(&mut rng, |item| item.1).unwrap().0 as usize)
    }

    pub fn generate(&self) -> String {
        lazy_static! {
            static ref end: Regex = Regex::new(r"[;:,\.!\?]+").unwrap();
        }

        let mut buff = String::new();
        let mut rng = rand::thread_rng();

        let mut current = self.start.choose(&mut rng).unwrap().clone() as usize;
        buff.push_str(&self.tokens[current]);
        while buff.len() < 330 {
            let next_id = self.pick_next(current);
            if next_id.is_none() {
                break;
            }

            current = next_id.unwrap();
            if !end.is_match(&self.tokens[current]) {
                buff.push_str(" ");
            }
            buff.push_str(&self.tokens[current]);
        }

        buff
    }

    fn read_header(file: &mut File) -> Result<i32> {
        let mut buf32: [u8; 4] = [0; 4];
        file.read_exact(&mut buf32)?;
        i32::unserialize(&buf32)
    }

    fn read_entry(file: &mut File) -> Result<String> {
        let mut buf8: [u8; 1] = [0; 1];
        let mut cstr: Vec<u8> = Vec::new();
        let mut reached_null = false;
        while !reached_null {
            file.read_exact(&mut buf8)?;
            if buf8[0] == 0 {
                reached_null = true;
            } else {
                cstr.push(buf8[0]);
            }
        }

        String::from_utf8(cstr).map_err(|e| Error::new_string_error(e.utf8_error()))
    }

    fn read_array(file: &mut File) -> Result<Vec<i32>> {
        let mut buf32: [u8; 4] = [0; 4];
        file.read_exact(&mut buf32)?;
        let size: usize = i32::unserialize(&buf32)? as usize;
        let mut array_buffer: Vec<u8> = vec![0; size * 4];
        file.read_exact(&mut array_buffer)?;

        Vec::unserialize(&array_buffer)
    }

    fn read_props(file: &mut File) -> Result<HashMap<i32, i32>> {
        let mut buf32: [u8; 4] = [0; 4];
        file.read_exact(&mut buf32)?;

        let len: usize = i32::unserialize(&buf32)? as usize;
        let mut buf: Vec<u8> = vec![0; len * 4 * 2];
        file.read_exact(&mut buf)?;

        HashMap::unserialize(&buf)
    }

    /// Unserialized a Markov chain from a binary file.
    pub fn from_binary(path: &Path) -> Result<MarkovChain> {
        let mut tokens: Vec<String> = Vec::new();
        let mut props: Vec<HashMap<i32, i32>> = Vec::new();

        let mut file = File::open(path)?;
        let counter = MarkovChain::read_header(&mut file).unwrap();
        for _ in 0..counter {
            let word = MarkovChain::read_entry(&mut file).unwrap();
            tokens.push(word);
        }

        let start: Vec<i32> = MarkovChain::read_array(&mut file).unwrap();
        let end: Vec<i32> = MarkovChain::read_array(&mut file).unwrap();

        while let Ok(prop) = MarkovChain::read_props(&mut file) {
            props.push(prop);
        }

        Ok(MarkovChain {
            tokens,
            props,
            start,
            end,
        })
    }

    /// Save a plain text version of the markov chain data into a file.
    pub fn save_txt(&self, path: &Path) -> Result<()> {
        let buff = self.txt_serialize();
        match File::create(path) {
            Ok(mut file) => file
                .write_all(buff.as_bytes())
                .map_err(|e| Error::new_io_error(e)),
            Err(w) => Err(Error::new_io_error(w)),
        }
    }

    /// Serialize the markov chain data to a plain text format.
    pub fn txt_serialize(&self) -> String {
        let mut buff = String::new();

        for word in self.tokens.iter() {
            buff.push_str(&format!("{};", word));
        }
        buff.push_str("\n");

        buff.push_str("start: [");
        for word in self.start.iter() {
            buff.push_str(&format!("{}, ", word));
        }
        buff.push_str("]\n");

        buff.push_str("end: [");
        for word in self.end.iter() {
            buff.push_str(&format!("{}, ", word));
        }
        buff.push_str("]\n");

        for (id, val) in self.props.iter().enumerate() {
            buff.push_str(&format!("{}: [", id));

            for (otherid, count) in val.iter() {
                buff.push_str(&format!("{} -> {}, ", otherid, *count,));
            }

            buff.push_str("]\n");
        }

        buff
    }

    /// Save a binary version of the markov chain data into a file.
    pub fn save_binary(&self, path: &Path) -> Result<()> {
        let ser = self.binary_serialize()?;
        match File::create(path) {
            Ok(mut bin_file) => bin_file.write_all(&ser).map_err(|e| Error::new_io_error(e)),
            Err(w) => Err(Error::new_io_error(w)),
        }
    }

    /// Serialize the markov chain data to a binary format.
    pub fn binary_serialize(&self) -> Result<Vec<u8>> {
        let words_count: i32 = self.tokens.len() as i32;
        let mut ser: Vec<u8> = Vec::new();
        ser.extend(&words_count.serialize()?);
        ser.extend(&self.tokens.serialize()?);

        ser.extend(&(self.start.len() as i32).serialize()?);
        ser.extend(&self.start.serialize()?);
        ser.extend(&(self.end.len() as i32).serialize()?);
        ser.extend(&self.end.serialize()?);

        for val in self.props.iter() {
            ser.extend(&(val.len() as i32).serialize()?);
            ser.extend(&val.serialize()?);
        }

        Ok(ser)
    }

    fn get_id(&self, word: &str) -> Option<i32> {
        match self.tokens.iter().position(|x| x == word) {
            Some(n) => Some(n as i32),
            None => None,
        }
    }

    fn increment_prop(id: i32, props: &mut HashMap<i32, i32>) {
        props.entry(id).and_modify(|e| *e += 1).or_insert(1);
    }

    /// Add a following word to a word or increment the number of time it follows it.
    pub fn add_props(&mut self, word: &str, next: &str) {
        let id = self.get_id(word).unwrap();
        let next_id = self.get_id(next).unwrap();

        MarkovChain::increment_prop(next_id, &mut self.props[id as usize]);
    }
}
