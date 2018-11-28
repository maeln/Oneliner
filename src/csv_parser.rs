extern crate crossbeam;

use csv::ReaderBuilder;

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
                    let cl = clean_line(line);
                    if url_filter(&cl) {
                        *line = "".to_string();
                    } else {
                        *line = cl;
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
    let words: Vec<&str> = line.split_whitespace().collect();
    for i in 0..words.len() {
        let word = words[i].to_string();

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
    line.to_lowercase().trim().to_string().replace("\0", "")
}

/// Filters
fn url_filter(line: &str) -> bool {
    line.contains("http://") || line.contains("https://")
}
