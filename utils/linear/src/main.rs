use std::io::{BufReader, BufRead};
use std::fs::File;
use nalgebra::{DVector, VectorN, DMatrix, MatrixN, U65};
use nalgebra::linalg::QR;
use std::collections::BTreeMap;


const NUM_ROWS :usize = 975_774;
const SCALE_AMT :f64 = 100_f64;

#[inline]
fn scale(x :f64) -> f64 {
    if x == 0f64 {
        x
    } else if x < 0f64 {
        x - SCALE_AMT
    } else {
        x + SCALE_AMT
    }
}

#[inline]
fn unscale(x :f64) -> f64 {
    if x == 0f64 {
        x
    } else if x < 0f64 {
        x + SCALE_AMT
    } else {
        x - SCALE_AMT
    }
}

// #[inline]
// fn unscale(x :f64) -> f64 {
//     if x < SCALE_AMT {
//         x + SCALE_AMT
//     } else if x > SCALE_AMT {
//         x - SCALE_AMT
//     } else {
//         x
//     }
// }

fn main() {
    let mut file = BufReader::new(File::open("/home/wspeirs/src/fishermann/data/rand_gen_1m.values").unwrap());

    let mut scores = Vec::with_capacity(NUM_ROWS);
    let mut values = Vec::with_capacity(NUM_ROWS * 65);

    println!("SCALE: {}", SCALE_AMT);

    // read in the first NUM_ROWS lines
    for _ in 0..NUM_ROWS {
        let mut line = String::new();
        file.read_line(&mut line);

        line.truncate(line.len()-1); // remove the newline

        let fields = line.split(":").collect::<Vec<_>>();

        // add on the score
        scores.push(scale(fields[0].parse::<f64>().unwrap()));

        values.extend(fields[1].split(" ").filter_map(|s| if s.is_empty() { None } else { Some( scale(s.parse::<f64>().unwrap()) ) } ));
    }

    println!("READ VALUES");
    // println!("{} {}", scores.len(), values.len());

    // let values = MatrixN::<f64, U65>::from_iterator(values.into_iter());
    // let values = DMatrix::from_iterator(NUM_ROWS, 65, values.into_iter());
    let values = DMatrix::from_vec(NUM_ROWS, 65, values);
    // let scores = VectorN::<f64, U65>::from_iterator(scores.into_iter());
    let scores = DVector::from_vec(scores);

    // println!("SCORES: {}", scores);
    // println!("VALUES: {}", values);

    let qr_decomp = values.clone().qr();

    let lls = qr_decomp.r().qr().try_inverse().unwrap() * qr_decomp.q().transpose() * scores.clone();

    println!("COMPUTED LLS");

    // println!("LLS: {}", lls);

    // go through each one, and compute how far off we are
    let computed_scores = values * lls;
    let mut diffs = Vec::with_capacity(NUM_ROWS);

    println!("COMPUTED SCORES");

    for (real_score, computed_score) in scores.iter().zip(computed_scores.iter().map(|s| unscale(*s))) {
        let diff = (real_score - computed_score).abs();
        // println!("{} {} {}", real_score, computed_score, diff);
        diffs.push(diff.floor() as u64);
    }

    let min = *diffs.iter().min().unwrap();
    let max = *diffs.iter().max().unwrap();
    let avg = diffs.iter().sum::<u64>() as f64 / NUM_ROWS as f64;

    diffs.sort_unstable();

    let p50 = diffs[(diffs.len() as f64 * 0.5f64) as usize];
    let p75 = diffs[(diffs.len() as f64 * 0.75f64) as usize];
    let p90 = diffs[(diffs.len() as f64 * 0.9f64) as usize];
    let p95 = diffs[(diffs.len() as f64 * 0.95f64) as usize];

    // create a histogram rounded to the nearest 100
    let mut histo = BTreeMap::<u64, u64>::new();

    for diff in diffs {
        let rounded_diff = (diff / 100) * 100;

        *histo.entry(rounded_diff).or_default() += 1;
    }

    println!();

    // let mut sorted_keys = histo.keys().collect::<Vec<_>>();
    // sorted_keys.sort_unstable();
    //
    // for key in sorted_keys {
    //     println!("{}: {}", key, histo[key]);
    // }

    println!("MIN: {} AVG: {} MAX: {} P50: {} P75: {} P90: {} P95: {}", min, avg, max, p50, p75, p90, p95);
}
