extern crate clap;
extern crate csv;
extern crate regex;

mod csv_parser;
mod markovchain;
mod serialize;

use clap::{App, Arg};
use std::path::Path;

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

    let mkc = csv_parser::parse_file(path);
    if mkc.save_binary(bin_path).is_err() {
        panic!("Could not save binary");
    } else {
        println!("Binary serialized in {}", bin_path.to_str().unwrap());
    }

    if mkc.save_txt(txt_path).is_err() {
        panic!("Could not save the txt");
    } else {
        println!("Text serialized in {}", txt_path.to_str().unwrap());
    }

    let mkc2 = markovchain::MarkovChain::from_binary(bin_path);
    if mkc2.is_err() {
        panic!("Could not load unserialized txt");
    } else {
        println!("Loaded from {}", bin_path.to_str().unwrap());
    }

    if mkc2.unwrap().save_txt(cmp_path).is_err() {
        panic!("Could not save unserialized txt");
    } else {
        println!("Binary serialized in {}", cmp_path.to_str().unwrap());
    }
}
