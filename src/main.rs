#[macro_use]
extern crate lazy_static;
extern crate clap;
extern crate crossbeam;
extern crate csv;
extern crate rand;
extern crate regex;

mod csv_parser;
mod markovchain;
mod serialize;

use clap::{App, Arg, SubCommand};
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
        .subcommand(
            SubCommand::with_name("parse")
                .arg(
                    Arg::with_name("text")
                        .short("t")
                        .help("Export to a text file instead of binary."),
                )
                .arg(
                    Arg::with_name("CSV_FILE")
                        .help("CSV file to use.")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("OUTPUT")
                        .help("Output file.")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("generate")
                .arg(
                    Arg::with_name("BIN_FILE")
                        .help("Markovchain binary file.")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("ONELINER_NUM")
                        .help("Number of oneliner to generate.")
                        .required(true)
                        .index(2),
                ),
        )
        .subcommand(
            SubCommand::with_name("convert")
                .arg(
                    Arg::with_name("input")
                        .help("input binary file.")
                        .short("-i")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("output")
                        .help("Output text file.")
                        .short("-o")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .get_matches();

    if let Some(sub_matches) = matches.subcommand_matches("parse") {
        let path = Path::new(sub_matches.value_of("CSV_FILE").unwrap());
        let bin_path = Path::new(sub_matches.value_of("OUTPUT").unwrap());
        let to_text = sub_matches.is_present("text");

        let mut now = Instant::now();
        let mkc = csv_parser::parse_file(path);
        println!("Parsed in {}s", get_fract_s(now),);

        now = Instant::now();
        if to_text {
            if mkc.save_txt(bin_path).is_err() {
                panic!("Could not save text file");
            } else {
                println!(
                    "Markovchain serialized in {}s in file: {}",
                    get_fract_s(now),
                    bin_path.to_str().unwrap()
                );
            }
        } else {
            if mkc.save_binary(bin_path).is_err() {
                panic!("Could not save binary");
            } else {
                println!(
                    "Binary serialized in {}s in file: {}",
                    get_fract_s(now),
                    bin_path.to_str().unwrap()
                );
            }
        }
    }

    if let Some(sub_matches) = matches.subcommand_matches("generate") {
        let bin_path = Path::new(sub_matches.value_of("BIN_FILE").unwrap());
        let num: usize = sub_matches
            .value_of("ONELINER_NUM")
            .unwrap()
            .parse()
            .unwrap();

        let now = Instant::now();
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
        let mkc = mkc2.unwrap();

        for _ in 0..num {
            println!("{}", mkc.generate());
            println!("--------------------------------------------------")
        }
    }

    if let Some(sub_matches) = matches.subcommand_matches("convert") {
        let bin_path = Path::new(sub_matches.value_of("input").unwrap());
        let text_path = Path::new(sub_matches.value_of("output").unwrap());

        let mut now = Instant::now();
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
        let mkc = mkc2.unwrap();

        now = Instant::now();
        let wrt_state = mkc.save_txt(text_path);
        if wrt_state.is_err() {
            panic!("Could serialize txt");
        } else {
            println!(
                "Serialize {} to {} in {}s",
                bin_path.to_str().unwrap(),
                text_path.to_str().unwrap(),
                get_fract_s(now),
            );
        }
    }
}
