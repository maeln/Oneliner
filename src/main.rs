extern crate csv;
extern crate regex;

use csv::ReaderBuilder;
use regex::Regex;

use std::collections::HashMap;

use std::cell::RefCell;
use std::rc::Rc;

use std::fs::File;
use std::io::Write;
use std::path::Path;

use std::mem;

fn main() {
    let path = Path::new("test.csv");
    let mut mkc = MarkovChain::new();
    mkc.parse_file(path);
}

fn to_le_bytes(num: i32) -> [u8; 4] {
    unsafe { mem::transmute(num.to_le()) }
}

fn float2byte(num: f32) -> [u8; 4] {
    unsafe { mem::transmute(num.to_bits().to_le()) }
}

pub struct MarkovChain {
    pub counter: i32,
    pub tokens: HashMap<String, i32>,
    pub props: Rc<RefCell<HashMap<i32, HashMap<i32, i32>>>>,
}

impl MarkovChain {
    pub fn new() -> MarkovChain {
        MarkovChain {
            counter: 0,
            tokens: HashMap::new(),
            props: Rc::new(RefCell::new(HashMap::new())),
        }
    }

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

        let ser = self.binary_serialize();
        let bin_path = Path::new("words.b");
        if let Ok(mut bin_file) = File::create(bin_path) {
            match bin_file.write_all(&ser) {
                Err(why) => println!("Error while writing binary: {}", why),
                Ok(_) => println!("Binary saved"),
            };
        }

        let words_path = Path::new("words.txt");
        if let Ok(mut word_file) = File::create(words_path) {
            let buff = self.txt_serialize();
            match word_file.write_all(buff.as_bytes()) {
                Err(why) => panic!("Error while writing word file: {}", why),
                Ok(_) => println!("Word file saved."),
            };
        }
    }

    pub fn txt_serialize(&self) -> String {
        let mut buff = String::new();
        for (word, id) in self.tokens.iter() {
            buff.push_str(&format!("{}:{};", word, id));
        }
        buff.push_str("\n");

        let props = self.props.borrow();
        for (id, val) in props.iter() {
            buff.push_str(&format!("{}: [", id));

            for (otherid, count) in val.iter() {
                buff.push_str(&format!("{} -> {}, ", otherid, count));
            }

            buff.push_str("]\n");
        }

        buff
    }

    pub fn binary_serialize(&self) -> Vec<u8> {
        let words_count: i32 = self.tokens.len() as i32;
        let mut ser: Vec<u8> = Vec::new();
        ser.extend_from_slice(&to_le_bytes(words_count));
        for (word, id) in self.tokens.iter() {
            ser.extend_from_slice(&to_le_bytes(*id));
            let wordc = word.clone();
            let mut bytes = wordc.into_bytes();
            bytes.push(b'\0');
            ser.append(&mut bytes);
        }

        let props = self.props.borrow();
        for (id, val) in props.iter() {
            ser.extend_from_slice(&to_le_bytes(*id));
            ser.extend_from_slice(&to_le_bytes(val.len() as i32));

            for (otherid, count) in val.iter() {
                ser.extend_from_slice(&to_le_bytes(*otherid));
                ser.extend_from_slice(&float2byte((*count as f32) / (val.len() as f32)))
            }
        }

        ser
    }

    fn clean_line(line: &String) -> String {
        line.to_lowercase()
    }

    fn get_id(&self, word: &String) -> i32 {
        *self.tokens.get(word).unwrap()
    }

    fn increment_prop(id: i32, props: &mut HashMap<i32, i32>) {
        let mut c = 0;
        {
            if props.contains_key(&id) {
                c = *props.get(&id).unwrap()
            } else {
                props.insert(id, 1);
            }
        }
        props.insert(id, c + 1);
    }

    fn add_props(&self, word: &String, next: &String) {
        let id = self.get_id(word);
        let next_id = self.get_id(next);
        let mut props_ref = self.props.borrow_mut();

        if props_ref.contains_key(&id) {
            let mut_prop = props_ref.get_mut(&id).unwrap();
            MarkovChain::increment_prop(next_id, mut_prop);
        } else {
            props_ref.insert(id, HashMap::new());
            props_ref.get_mut(&id).unwrap().insert(next_id, 1);
        }
    }

    fn get_words(&mut self, line: &String) {
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
