use std::env;
use std::io::BufReader;
use std::fs::File;
use std::mem;

use rayon::prelude::*;
use memmap::MmapOptions;
use smallvec::{smallvec, SmallVec};
use std::collections::HashMap;

enum CastleType {
    KING,
    QUEEN
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
enum Elo {
    Beginner,
    Intermediate,
    Advanced,
    Master
}

/// 0 - 1200; 1201 - 1800; 1801 - 2400; >2400
#[inline]
fn bucket_elo(elo :usize) -> Elo {
    if elo <= 1200 {
        Elo::Beginner
    } else if elo <= 1800 {
        Elo::Intermediate
    } else if elo <= 2400 {
        Elo::Advanced
    } else {
        Elo::Master
    }
}

/*
 * Goes through a CSV file created by pgn2csv searching for moves, etc
 */
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Must supply the CSV file on the command line");
        return;
    }

    let csv_file = File::open(&args[1]).expect("Error opening CSV file");
    let csv_mmap = unsafe { MmapOptions::new().map(&csv_file).expect("Error creating mmap") };

    // we want to skip the header line
    let offset = csv_mmap.as_ref().iter().position(|c| *c == 0x0A).expect("Cannot find first newline") as usize + 1;

    let mmap_string = unsafe { String::from_raw_parts(csv_mmap.as_ptr().add(offset) as *mut u8, csv_mmap.len() - offset, csv_mmap.len() - offset) };

    let results = mmap_string.par_lines().map(|line| {
        let fields = line.split(",").collect::<Vec<_>>();
        let moves = fields.last().unwrap().split(";").collect::<Vec<_>>(); // the last field is always the moves

        // println!("|{}| |{}|", fields[2], fields[3]);

        let white_elo = fields[2].parse::<usize>().expect(format!("Error parsing 2: |{}|", fields[2]).as_str());
        let black_elo = fields[3].parse::<usize>().expect(format!("Error parsing 3: |{}|", fields[3]).as_str());

        // setup our small vec to hold the castling: white, black
        let mut ret :SmallVec<[(Elo, Option<CastleType>); 2]> = smallvec![(bucket_elo(white_elo), None), (bucket_elo(black_elo), None)];

        for (i, mv) in moves.iter().enumerate() {
            let mut r : &mut (Elo, Option<CastleType>) = &mut ret[i % 2];

            if *mv == "O-O" {
                r.1 = Some(CastleType::KING);
            } else if *mv == "O-O-O" {
                r.1 = Some(CastleType::QUEEN);
            }
        }

        ret
    }).collect::<Vec<_>>();

    // we don't want this memory to be freed
    mem::forget(mmap_string);

    // let mut histo = HashMap::new();

    let total = results.iter().filter(|res| res[0].0 == Elo::Master && res[1].0 == Elo::Master).count() as f64;

    // count how many times white & black castle
    let (w_count, b_count) = results.iter().filter(|res| res[0].0 == Elo::Master && res[1].0 == Elo::Master).fold((0_u64,0_u64), |(w,b), res| {
        (if res[0].1.is_some() { w+1 } else { w }, if res[1].1.is_some() { b+1 } else { b })
    });

    println!("CASTLES WHITE: {:02}%\tBLACK: {:02}%", (w_count as f64 / total) * 100.0, (b_count as f64 / total) * 100.0);
}