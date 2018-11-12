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
    pub counter: i32,
    pub tokens: HashMap<String, i32>,
    pub props: HashMap<i32, HashMap<i32, i32>>,
}

impl MarkovChain {
    pub fn new() -> MarkovChain {
        MarkovChain {
            counter: 0,
            tokens: HashMap::new(),
            props: HashMap::new(),
        }
    }

    fn read_header(file: &mut File) -> Result<i32, ()> {
        let mut buf32: [u8; 4] = [0; 4];
        match file.read_exact(&mut buf32) {
            Ok(_) => Ok(lebytestoi32(buf32)),
            Err(_) => Err(()),
        }
    }

    fn read_entry(file: &mut File) -> Result<(i32, String), &str> {
        let mut buf32: [u8; 4] = [0; 4];

        let idres = file.read_exact(&mut buf32);
        if idres.is_err() {
            return Err("Could not read entry ID.");
        }
        let id = lebytestoi32(buf32);

        let mut buf8: [u8; 1] = [0; 1];
        let mut cstr: Vec<u8> = Vec::new();
        let mut reached_null = false;
        while !reached_null {
            let idres = file.read_exact(&mut buf8);
            if idres.is_err() {
                return Err("Could not read entry word.");
            }
            if buf8[0] == 0 {
                reached_null = true;
            } else {
                cstr.push(buf8[0]);
            }
        }

        let resword = String::from_utf8(cstr);
        if resword.is_err() {
            return Err("Unable to convert word to UTF-8.");
        }
        let word = resword.unwrap();

        Ok((id, word))
    }

    /// Unserialized a Markov chain from a binary file.
    pub fn from_binary(path: &Path) -> Result<MarkovChain, &str> {
        let mut tokens: HashMap<String, i32> = HashMap::new();
        let mut props: HashMap<i32, HashMap<i32, i32>> = HashMap::new();

        let fres = File::open(path);
        if fres.is_err() {
            return Err("Impossible to open file");
        }
        let mut file = fres.unwrap();

        let counter = MarkovChain::read_header(&mut file).unwrap();
        for _ in 0..counter {
            let (id, word) = MarkovChain::read_entry(&mut file).unwrap();
            tokens.insert(word, id);
        }

        Ok(MarkovChain {
            counter,
            tokens,
            props,
        })
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
        for (word, id) in self.tokens.iter() {
            buff.push_str(&format!("{}:{};", word, id));
        }
        buff.push_str("\n");

        for (id, val) in self.props.iter() {
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
        for (word, id) in self.tokens.iter() {
            ser.extend_from_slice(&i32tolebytes(*id));
            let wordc = word.clone();
            let mut bytes = wordc.into_bytes();
            bytes.push(b'\0');
            ser.append(&mut bytes);
        }

        for (id, val) in self.props.iter() {
            ser.extend_from_slice(&i32tolebytes(*id));
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

    fn get_id(&self, word: &str) -> i32 {
        self.tokens[word]
    }

    fn increment_prop(id: i32, props: &mut HashMap<i32, i32>) {
        props.entry(id).and_modify(|e| *e += 1).or_insert(1);
    }

    /// Add a following word to a word or increment the number of time it follows it.
    fn add_props(&mut self, word: &str, next: &str) {
        let id = self.get_id(word);
        let next_id = self.get_id(next);

        self.props
            .entry(id)
            .and_modify(|e| MarkovChain::increment_prop(next_id, e))
            .or_insert_with(HashMap::new)
            .insert(next_id, 1);
    }

    /// Get all the words in a oneliner.
    fn get_words(&mut self, line: &str) {
        let mut words: Vec<String> = Vec::new();
        let re = Regex::new(r"\w+").unwrap();
        for cap in re.captures_iter(line) {
            words.push(cap[0].to_string());
        }

        for word in words.iter() {
            if !self.tokens.contains_key(word) {
                self.tokens.insert(word.to_string(), self.counter);
                self.counter += 1;
            }
        }

        for i in 0..words.len() {
            if i < (words.len() - 1) {
                self.add_props(&words[i], &words[i + 1]);
            }
        }
    }
}
