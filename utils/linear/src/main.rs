use std::io::{BufReader, BufRead};
use std::fs::File;
use nalgebra::{VectorN, DMatrix, MatrixN, U65};

fn main() {
    let mut file = BufReader::new(File::open("/home/wspeirs/src/fishermann/data/rand_gen_1m.values").unwrap());

    let mut scores = Vec::new();
    let mut values = Vec::new();

    // read in the first 65 lines
    for _ in 0..65 {
        let mut line = String::new();
        file.read_line(&mut line);

        line.truncate(line.len()-1); // remove the newline

        let fields = line.split(":").collect::<Vec<_>>();

        // add on the score
        scores.push(fields[0].parse::<f64>().unwrap());

        values.extend(fields[1].split(" ").filter_map(|s| if s.is_empty() { None } else { Some(s.parse::<f64>().unwrap()) } ));
    }

    println!("{} {}", scores.len(), values.len());

    let values = MatrixN::<f64, U65>::from_iterator(values.into_iter());
    // let values = DMatrix::from_iterator(65, 65, values.into_iter());
    let scores = VectorN::<f64, U65>::from_iterator(scores.into_iter());

    let coefficients = values.qr().solve(&scores).unwrap();

    println!("SCORES: {}", scores);
    println!("VALUES: {}", values);
    println!("COEFF: {}", coefficients);
}
