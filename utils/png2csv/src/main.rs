use std::env;
use std::io::{BufReader, BufWriter, Write, BufRead};
use std::fs::{File, OpenOptions};
use std::path::Path;
use regex::Regex;
use std::time::Instant;

/*
 * Converts a PGN file from lichess into a csv file of variable widths
 */
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Must supply the PGN file on the command line");
        return;
    }

    let csv_file_path = args[1].replace("pgn", "csv");
    let mut pgn_file = BufReader::new(File::open(&args[1]).expect("Error opening PGN file"));
    let mut csv_file = BufWriter::new(OpenOptions::new().truncate(true).write(true).create(true).open(csv_file_path).expect("Error opening CSV file"));

    // write out the pseudo-header for the CSV file
    writeln!(csv_file, "white, white-elo, black, black-elo, time-control");

    let mut line = String::new();
    let mut headers = vec!["".to_string(); 5];

    let white_pattern = Regex::new(r#"^\[White\s+"(.+)"\]"#).unwrap();
    let black_pattern = Regex::new(r#"^\[Black\s+"(.+)"\]"#).unwrap();
    let white_elo_pattern = Regex::new(r#"^\[WhiteElo\s+"(.+)"\]"#).unwrap();
    let black_elo_pattern = Regex::new(r#"^\[BlackElo\s+"(.+)"\]"#).unwrap();
    let time_control_pattern = Regex::new(r#"^\[TimeControl\s+"(.+)"\]"#).unwrap();

    let clock_pattern = Regex::new(r#" \{ \[%clk \d+:\d+:\d+\] \} "#).unwrap();
    let index_pattern = Regex::new(r#"\d+\. "#).unwrap();

    let mut count = 0;
    // let mut start = Instant::now();

    // go through the file reading all the lines
    while let Ok(len) = pgn_file.read_line(&mut line) {
        if len == 0 {
            break
        }
        
        if white_pattern.is_match(line.as_str()) {
            // println!("FOUND WHITE: {:?}", white_pattern.captures(line.as_str()).unwrap());
            headers[0] = white_pattern.captures(line.as_str()).unwrap()[1].to_string();
        } else if black_pattern.is_match(line.as_str()) {
            headers[1] = black_pattern.captures(line.as_str()).unwrap()[1].to_string();
        } else if white_elo_pattern.is_match(line.as_str()) {
            headers[2] = white_elo_pattern.captures(line.as_str()).unwrap()[1].to_string();
        } else if black_elo_pattern.is_match(line.as_str()) {
            headers[3] = black_elo_pattern.captures(line.as_str()).unwrap()[1].to_string();
        } else if time_control_pattern.is_match(line.as_str()) {
            headers[4] = time_control_pattern.captures(line.as_str()).unwrap()[1].to_string();
        } else if line.starts_with("1.") {
            // first print the headers
            csv_file.write_all(headers.join(",").as_bytes());
            csv_file.write_all(",".as_bytes());

            let no_clks = clock_pattern.replace_all(line.as_str(), ",");
            // println!("NO CLKS: {}", no_clks);

            let moves = index_pattern.replace_all(no_clks.as_ref(), "");

            csv_file.write_all(moves.as_bytes());

            count += 1;

            if count % 100_000 == 0 {
                // let gps = 100_000.0 / start.elapsed().as_secs_f64();
                // println!("{:.02} games/sec", gps);
                // start = Instant::now();

                println!("{} games", count);
            }

            // if count > 1_000_000 {
            //     return
            // }
        }

        line.clear();
    }
}
