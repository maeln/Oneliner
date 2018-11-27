use std::collections::HashMap;

use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use serialize;

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

    fn read_header(file: &mut File) -> Result<i32, String> {
        let mut buf32: [u8; 4] = [0; 4];
        file.read_exact(&mut buf32)
            .map_err(|e| format!("Could not read header: {}", e))?;
        Ok(serialize::lebytestoi32(buf32))
    }

    fn read_entry(file: &mut File) -> Result<String, String> {
        let mut buf8: [u8; 1] = [0; 1];
        let mut cstr: Vec<u8> = Vec::new();
        let mut reached_null = false;
        while !reached_null {
            file.read_exact(&mut buf8)
                .map_err(|e| format!("Could not read entry: {}", e))?;
            if buf8[0] == 0 {
                reached_null = true;
            } else {
                cstr.push(buf8[0]);
            }
        }

        let resword = String::from_utf8(cstr)
            .map_err(|e| format!("Could not convert entry to string: {}", e))?;
        Ok(resword)
    }

    fn read_array(file: &mut File) -> Result<Vec<i32>, String> {
        let mut buf32: [u8; 4] = [0; 4];
        file.read_exact(&mut buf32)
            .map_err(|e| format!("Could not read array size: {}", e))?;
        let size: i32 = serialize::lebytestoi32(buf32);
        let mut arr: Vec<i32> = Vec::with_capacity(size as usize);
        for i in 0..size {
            file.read_exact(&mut buf32)
                .map_err(|e| format!("Could not read array item at position {}: {}", i, e))?;
            arr.push(serialize::lebytestoi32(buf32));
        }

        Ok(arr)
    }

    fn read_pair(file: &mut File) -> Result<(i32, i32), String> {
        let mut buf64: [u8; 8] = [0; 8];
        file.read_exact(&mut buf64)
            .map_err(|e| format!("Could not read pair of i32: {}", e))?;
        let id = serialize::lebytestoi32([buf64[0], buf64[1], buf64[2], buf64[3]]);
        let count = serialize::lebytestoi32([buf64[4], buf64[5], buf64[6], buf64[7]]);
        Ok((id, count))
    }

    fn read_props(file: &mut File) -> Result<HashMap<i32, i32>, std::io::Error> {
        let mut props: HashMap<i32, i32> = HashMap::new();
        let mut buf32: [u8; 4] = [0; 4];
        let buf_read = file.read_exact(&mut buf32);

        if buf_read.is_ok() {
            let len = serialize::lebytestoi32(buf32);
            for _ in 0..len {
                let (id, count) = MarkovChain::read_pair(file).unwrap();
                props.insert(id, count);
            }
        } else {
            return Err(buf_read.err().unwrap());
        }

        Ok(props)
    }

    /// Unserialized a Markov chain from a binary file.
    pub fn from_binary(path: &Path) -> Result<MarkovChain, String> {
        let mut tokens: Vec<String> = Vec::new();
        let mut props: Vec<HashMap<i32, i32>> = Vec::new();

        let mut file = File::open(path)
            .map_err(|e| format!("Could not open file {} : {}", path.to_str().unwrap(), e))?;
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
    pub fn save_txt(&self, path: &Path) -> Result<(), std::io::Error> {
        let buff = self.txt_serialize();
        match File::create(path) {
            Ok(mut file) => file.write_all(buff.as_bytes()),
            Err(w) => Err(w),
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
    pub fn save_binary(&self, path: &Path) -> Result<(), std::io::Error> {
        let ser = self.binary_serialize();
        match File::create(path) {
            Ok(mut bin_file) => bin_file.write_all(&ser),
            Err(w) => Err(w),
        }
    }

    /// Serialize the markov chain data to a binary format.
    pub fn binary_serialize(&self) -> Vec<u8> {
        let words_count: i32 = self.tokens.len() as i32;
        let mut ser: Vec<u8> = Vec::new();
        ser.extend(&serialize::i32tolebytes(words_count));
        ser.extend(serialize::string_list_to_bytes(&self.tokens));

        ser.extend(&serialize::i32tolebytes(self.start.len() as i32));
        ser.extend(&serialize::i32_list_to_bytes(&self.start));

        ser.extend(&serialize::i32tolebytes(self.end.len() as i32));
        ser.extend(&serialize::i32_list_to_bytes(&self.end));

        for val in self.props.iter() {
            ser.extend(&serialize::i32tolebytes(val.len() as i32));
            ser.extend(serialize::i32_hash_to_bytes(&val));
        }

        ser
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
