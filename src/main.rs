extern crate csv;
extern crate regex;

use csv::ReaderBuilder;
use regex::Regex;

use std::collections::HashMap;
use std::collections::LinkedList;

use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let path = Path::new("test.csv");
    let mut mkc = MarkovChain::new();
    mkc.parse_file(path);
}

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

        let words_path = Path::new("words.txt");
        if let Ok(mut word_file) = File::create(words_path) {
            let mut buff = String::new();
            for (word, id) in self.tokens.iter() {
                buff.push_str(&format!("{}:{};", word, id));
            }
            buff.push_str("\n");

            match word_file.write_all(buff.as_bytes()) {
                Err(why) => panic!("Error while writing word file: {}", why),
                Ok(_) => println!("Word file saved."),
            };
        }
    }

    fn clean_line(line: &String) -> String {
        line.to_lowercase()
    }

    fn get_words(&mut self, line: &String) {
        let mut words: LinkedList<String> = LinkedList::new();
        let re = Regex::new(r"\w+").unwrap();
        for cap in re.captures_iter(line) {
            words.push_back(cap[0].to_string());
        }

        for word in words.iter() {
            if !self.tokens.contains_key(word) {
                self.tokens.insert(word.to_string(), self.counter);
                self.counter += 1;
            }
        }
    }
}
