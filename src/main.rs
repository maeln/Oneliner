extern crate clap;
extern crate csv;

mod csv_parser;
mod markovchain;
mod serialize;

use clap::{App, Arg};
use std::path::Path;
use std::time::{Duration, Instant};

fn get_fract_s(date: Instant) -> String {
    let duration: Duration = date.elapsed();
    format!("{}.{:0>3}", duration.as_secs(), duration.subsec_millis())
}

fn main() {
    let matches = App::new("Oneliner")
        .version("0.1a")
        .author("Maeln <contact@maeln.com>")
        .arg(
            Arg::with_name("CSV_FILE")
                .help("CSV file to use.")
                .required(true)
                .index(1),
        )
        .get_matches();

    let path = Path::new(matches.value_of("CSV_FILE").unwrap());
    let bin_path = Path::new("words.b");
    let txt_path = Path::new("words.txt");
    let cmp_path = Path::new("comp.txt");

    println!("Starting to parse {}", path.to_str().unwrap());
    let mut now = Instant::now();
    let mkc = csv_parser::parse_file(path);
    println!("Parsed in {}s", get_fract_s(now),);

    now = Instant::now();
    if mkc.save_binary(bin_path).is_err() {
        panic!("Could not save binary");
    } else {
        println!(
            "Binary serialized in {}s in file: {}",
            get_fract_s(now),
            bin_path.to_str().unwrap()
        );
    }

    now = Instant::now();
    if mkc.save_txt(txt_path).is_err() {
        panic!("Could not save the txt");
    } else {
        println!(
            "Text serialized in {}s in file: {}",
            get_fract_s(now),
            txt_path.to_str().unwrap()
        );
    }

    now = Instant::now();
    let mkc2 = markovchain::MarkovChain::from_binary(bin_path);
    if mkc2.is_err() {
        panic!("Could not load unserialized txt");
    } else {
        println!(
            "Unserialized binary from {} in {}s",
            bin_path.to_str().unwrap(),
            get_fract_s(now),
        );
    }

    now = Instant::now();
    if mkc2.unwrap().save_txt(cmp_path).is_err() {
        panic!("Could not save unserialized txt");
    } else {
        println!(
            "Text serialized in {}s in file: {}",
            get_fract_s(now),
            cmp_path.to_str().unwrap()
        );
    }
}
