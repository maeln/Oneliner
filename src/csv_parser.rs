use csv::ReaderBuilder;
use regex::Regex;

use markovchain::MarkovChain;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use std::time::Instant;

fn get_fract_s(date: Instant) -> String {
    let duration = date.elapsed();
    format!("{}.{:0>3}", duration.as_secs(), duration.subsec_millis())
}

/// Make a corpus from the CSV
pub fn csv_to_corpus(path: &Path) -> Vec<String> {
    let fname = path.display();
    let mut corpus: Vec<String> = Vec::new();

    println!("Reading the CSV... ");
    let now = Instant::now();

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
                corpus.push(line);
            }
        }
    }

    println!("CSV file read in {}", get_fract_s(now));

    corpus
}

/// Clean a corpus
pub fn clean_corpus(corpus: &mut [String]) {
    println!("Cleaning the corpus... ");
    let now = Instant::now();

    // Number of thread to use. TODO: Should be determined at runtime.
    let thread_num = 4;

    // Divide the corpus by the number of thread
    let dist = corpus.len() / thread_num;

    let res = crossbeam::scope(|scope| {
        for slice in corpus.chunks_mut(dist) {
            scope.spawn(move |_| {
                for line in slice.iter_mut() {
                    if filter_line(line) {
                        *line = "".to_string();
                    } else {
                        *line = clean_line(line);
                    }
                }
            });
        }
    });

    if res.is_err() {
        panic!("Could not parse corpus.");
    }

    println!("Corpus cleaned in {}", get_fract_s(now));
}

/// Parse a oneliner CSV and make it into a markov chain.
pub fn parse_file(path: &Path) -> MarkovChain {
    let mut chain = MarkovChain::new();

    let mut corpus = csv_to_corpus(path);
    clean_corpus(&mut corpus);
    for line in corpus.iter() {
        get_words(&mut chain, &line);
    }

    chain
}

/// Get all the words in a oneliner.
fn get_words(chain: &mut MarkovChain, line: &str) {
    lazy_static! {
        static ref split_word_re: Regex = Regex::new(r"\s+").unwrap();
    }
    let words: Vec<&str> = split_word_re.split(line).collect();
    for i in 0..words.len() {
        let word = words[i].trim().to_string();

        if !chain.tokens.contains(&word) {
            chain.tokens.push(word.clone());
            chain.props.push(HashMap::new());
        }

        let id = (chain.tokens.len() - 1) as i32;
        if i == 0 && !chain.start.contains(&id) {
            chain.start.push(id);
        }

        if i == (words.len() - 1) && !chain.end.contains(&id) {
            chain.end.push(id);
        }

        if i > 0 {
            chain.add_props(&words[i - 1], &words[i]);
        }
    }
}

/// Clean the text of a line.
fn clean_line(line: &str) -> String {
    lazy_static! {
        static ref multiple_ponct: Regex = Regex::new(r"(?P<unspaced>[;:\.!\?]+)").unwrap();
    }

    let cleaned_line = line.to_lowercase().trim().to_string().replace("\0", "");
    let rm_mlponct = multiple_ponct.replace_all(&cleaned_line, " $unspaced ");

    rm_mlponct.to_string()
}

/// Filters
fn filter_line(line: &str) -> bool {
    url_filter(line) || no_char_filter(line) || no_hashtag_bullshit(line) || ascii_filter(line)
}

/// Only accept ascii strings
fn ascii_filter(line: &str) -> bool {
    !line.is_ascii()
}

/// Filters oneliner containing urls
fn url_filter(line: &str) -> bool {
    lazy_static! {
        static ref url_reg: Regex = Regex::new(r"(.+://.+\.[a-z]+.*$)|(.*www\..*)").unwrap();
    }
    url_reg.is_match(line)
}

/// Filters oneliner that don't contain any [a-Z] char.
fn no_char_filter(line: &str) -> bool {
    lazy_static! {
        static ref char_reg: Regex = Regex::new(r".*[a-zA-Z]+.*").unwrap();
    }
    !char_reg.is_match(line)
}

/// Filter ### bullshit
fn no_hashtag_bullshit(line: &str) -> bool {
    lazy_static! {
        static ref bull_reg: Regex = Regex::new(r"^#+").unwrap();
    }

    bull_reg.is_match(line)
}
