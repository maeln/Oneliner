extern crate csv;
extern crate regex;

use csv::ReaderBuilder;
use regex::Regex;

use std::collections::HashMap;

use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use std::mem;

fn main() {
    let path = Path::new("test.csv");
    let bin_path = Path::new("words.b");
    let txt_path = Path::new("words.txt");
    let cmp_path = Path::new("comp.txt");

    let mut mkc = MarkovChain::new();
    mkc.parse_file(path);
    if mkc.save_binary(bin_path).is_err() {
        panic!("Could not save binary");
    }

    if mkc.save_txt(txt_path).is_err() {
        panic!("Could not save the txt");
    }

    let mkc2 = MarkovChain::from_binary(bin_path).unwrap();
    if mkc2.save_txt(cmp_path).is_err() {
        panic!("Could not save unserialized txt");
    }
}

fn i32tolebytes(num: i32) -> [u8; 4] {
    unsafe { mem::transmute(num.to_le()) }
}

fn lebytestoi32(num: [u8; 4]) -> i32 {
    unsafe { mem::transmute::<[u8; 4], i32>(num).to_le() }
}

#[derive(Default)]
pub struct MarkovChain {
    pub tokens: Vec<String>,
    pub props: Vec<HashMap<i32, i32>>,
}

impl MarkovChain {
    pub fn new() -> MarkovChain {
        MarkovChain {
            tokens: Vec::new(),
            props: Vec::new(),
        }
    }

    fn read_header(file: &mut File) -> Result<i32, String> {
        let mut buf32: [u8; 4] = [0; 4];
        file.read_exact(&mut buf32)
            .map_err(|e| format!("Could not read header: {}", e))?;
        Ok(lebytestoi32(buf32))
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

    fn read_pair(file: &mut File) -> Result<(i32, i32), String> {
        let mut buf64: [u8; 8] = [0; 8];
        file.read_exact(&mut buf64)
            .map_err(|e| format!("Could not read pair of i32: {}", e))?;
        let id = lebytestoi32([buf64[0], buf64[1], buf64[2], buf64[3]]);
        let count = lebytestoi32([buf64[4], buf64[5], buf64[6], buf64[7]]);
        Ok((id, count))
    }

    fn read_props(file: &mut File) -> Result<HashMap<i32, i32>, std::io::Error> {
        let mut props: HashMap<i32, i32> = HashMap::new();
        let mut buf32: [u8; 4] = [0; 4];
        let buf_read = file.read_exact(&mut buf32);

        if buf_read.is_ok() {
            let len = lebytestoi32(buf32);
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

        while let Ok(prop) = MarkovChain::read_props(&mut file) {
            props.push(prop);
        }

        Ok(MarkovChain { tokens, props })
    }

    /// Parse a oneliner CSV and make it into a markov chain.
    pub fn parse_file(&mut self, path: &Path) {
        let fname = path.display();

        println!("Parsing '{}'", fname);

        let file = match File::open(&path) {
            Err(why) => panic!("{} couldn't be read: {}", fname, why),
            Ok(file) => file,
        };

        let mut parser = ReaderBuilder::new()
            .delimiter(b';')
            .flexible(true)
            .from_reader(file);

        for oneliner in parser.records() {
            match oneliner {
                Err(why) => println!("One record failed: {}", why),
                Ok(record) => {
                    if record.len() < 5 {
                        println!("Row has less than 5 row, skipping.");
                        continue;
                    }
                    let mut line: String = String::new();
                    for i in 4..record.len() {
                        if let Some(sentence) = record.get(i) {
                            line.push_str(sentence);
                        }
                    }
                    line = MarkovChain::clean_line(&line);
                    self.get_words(&line);
                }
            }
        }

        println!("Finished parsing.");
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
        ser.extend_from_slice(&i32tolebytes(words_count));
        for word in self.tokens.iter() {
            let wordc = word.clone();
            let mut bytes = wordc.into_bytes();
            bytes.push(b'\0');
            ser.append(&mut bytes);
        }

        for val in self.props.iter() {
            ser.extend_from_slice(&i32tolebytes(val.len() as i32));

            for (otherid, count) in val.iter() {
                ser.extend_from_slice(&i32tolebytes(*otherid));
                ser.extend_from_slice(&i32tolebytes(*count));
            }
        }

        ser
    }

    /// Clean the text of a line.
    fn clean_line(line: &str) -> String {
        line.to_lowercase()
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
    fn add_props(&mut self, word: &str, next: &str) {
        let id = self.get_id(word).unwrap();
        let next_id = self.get_id(next).unwrap();

        MarkovChain::increment_prop(next_id, &mut self.props[id as usize]);
    }

    /// Get all the words in a oneliner.
    fn get_words(&mut self, line: &str) {
        let mut words: Vec<String> = Vec::new();
        let re = Regex::new(r"\w+").unwrap();
        for cap in re.captures_iter(line) {
            words.push(cap[0].to_string());
        }

        for word in words.iter() {
            if !self.tokens.contains(&word) {
                self.tokens.push(word.clone());
                self.props.push(HashMap::new());
            }
        }

        for i in 0..words.len() {
            if i < (words.len() - 1) {
                self.add_props(&words[i], &words[i + 1]);
            }
        }
    }
}
