use csv::ReaderBuilder;

use markovchain::MarkovChain;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

/// Parse a oneliner CSV and make it into a markov chain.
pub fn parse_file(path: &Path) -> MarkovChain {
    let fname = path.display();
    let mut chain = MarkovChain::new();

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
                line = clean_line(&line);
                get_words(&mut chain, &line);
            }
        }
    }

    println!("Finished parsing.");

    chain
}

/// Get all the words in a oneliner.
fn get_words(chain: &mut MarkovChain, line: &str) {
    let mut words: Vec<String> = Vec::new();
    let re = Regex::new(r"\w+").unwrap();
    for cap in re.captures_iter(line) {
        words.push(cap[0].to_string());
    }

    for word in words.iter() {
        if !chain.tokens.contains(&word) {
            chain.tokens.push(word.clone());
            chain.props.push(HashMap::new());
        }
    }

    for i in 0..words.len() {
        if i < (words.len() - 1) {
            chain.add_props(&words[i], &words[i + 1]);
        }
    }
}

/// Clean the text of a line.
fn clean_line(line: &str) -> String {
    line.to_lowercase()
}
