use bloomfilter::*;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::path::PathBuf;

pub fn read_input(path: &PathBuf) -> Result<Vec<String>, io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut input = Vec::new();
    for line in reader.lines() {
        match line {
            Ok(l) => input.push(l),
            Err(e) => return Err(e),
        }
    }
    Ok(input)
}

pub fn write_to_file() {}

// pub fn check_bloom(path: &PathBuf, input: Vec<String>) -> Result<Vec<String>, io::Error> {
//     let bloom_bytes = load_bloom(path)?;
// }

pub fn load_bloom(path: &PathBuf) -> Result<Vec<u8>, io::Error> {
    let bytes = std::fs::read(path)?;
    Ok(bytes)
}

pub fn create_bloom(
    input: Vec<String>,
    output_path: &PathBuf,
    size: &usize,
    positive_rate: &f64,
) -> Result<(), String> {
    Ok(())
}

pub fn create_bloom_from_file(
    input_path: &PathBuf,
    output_path: &PathBuf,
    positive_rate: &f64,
) -> Result<(), String> {
    let input = match read_input(input_path) {
        Ok(input) => input,
        Err(e) => return Err(format!("{}: {}", input_path.display(), e)),
    };
    let size = &input.len();
    create_bloom(input, output_path, size, positive_rate)
}

pub fn create_bloom_from_queryhash() {}
