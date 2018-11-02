extern crate csv;
extern crate regex;

use csv::ReaderBuilder;
use regex::Regex;

use std::collections::LinkedList;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let path = Path::new("oneliner.csv");
    let fname = path.display();

    println!("Parsing '{}'", fname);

    let file = match File::open(&path) {
        Err(why) => panic!("{} couldn't be read: {}", fname, why),
        Ok(file) => file,
    };

    let mut words: LinkedList<String> = LinkedList::new();
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
                line = clean_line(&line);
                let line_words = get_words(&line);
                for word in line_words.iter() {
                    if !words.contains(word) {
                        words.push_back(word.to_string());
                    }
                }
            }
        }
    }

    let words_path = Path::new("words.txt");
    if let Ok(mut word_file) = File::create(words_path) {
        let mut buff = String::new();
        for word in words.iter() {
            buff.push_str(&format!("{}\n", word));
        }

        match word_file.write_all(buff.as_bytes()) {
            Err(why) => panic!("Error while writing word file: {}", why),
            Ok(_) => println!("Word file saved."),
        };
    }
}

fn clean_line(line: &String) -> String {
    line.to_lowercase()
}

fn get_words(line: &String) -> LinkedList<String> {
    let mut words: LinkedList<String> = LinkedList::new();
    let re = Regex::new(r"([^\s])+").unwrap();
    for cap in re.captures_iter(line) {
        words.push_back(cap[0].to_string());
    }

    words
}
