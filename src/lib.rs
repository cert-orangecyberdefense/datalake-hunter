use bloomfilter::Bloom;
// use serde::Deserialize;
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

pub fn load_bloom(path: &PathBuf) -> Result<(), String> {
    let ron_string = match std::fs::read_to_string(path) {
        Ok(ron_string) => ron_string,
        Err(e) => return Err(format!("{}: {}", path.display(), e)),
    };

    let _bloom: Bloom<String> = match ron::from_str(&ron_string) {
        Ok(bloom) => bloom,
        Err(_) => {
            return Err(format!(
                "Failed to deserialize bloom filter located in {}",
                path.display()
            ))
        }
    };
    Ok(())
}

pub fn create_bloom(input: Vec<String>, size: usize, positive_rate: f64) -> Bloom<String> {
    let mut bloom: Bloom<String> = Bloom::new_for_fp_rate(size, positive_rate);
    for value in input {
        bloom.set(&value);
    }
    bloom
}

pub fn write_bloom_to_file(bloom: Bloom<String>, output_path: &PathBuf) -> Result<(), String> {
    let bloom_ron = ron::to_string(&bloom).expect("Failed to serialize the bloomfilter");
    let mut output = match File::create(output_path) {
        Ok(output) => output,
        Err(e) => return Err(format!("{}: {}", output_path.display(), e)),
    };
    match write!(output, "{}", bloom_ron) {
        Ok(()) => (),
        Err(e) => return Err(format!("{}: {}", output_path.display(), e)),
    }
    Ok(())
}

pub fn create_bloom_from_file(
    input_path: &PathBuf,
    output_path: &PathBuf,
    positive_rate: f64,
) -> Result<(), String> {
    let input = match read_input(input_path) {
        Ok(input) => input,
        Err(e) => return Err(format!("{}: {}", input_path.display(), e)),
    };
    let size = input.len();

    let bloom = create_bloom(input, size, positive_rate);
    write_bloom_to_file(bloom, output_path)
}

pub fn create_bloom_from_queryhash() {}

#[test]
fn test_bloom_serialization() {
    let values: Vec<String> = vec![
        "test1".to_string(),
        "test2".to_string(),
        "test3".to_string(),
    ];
    let size: usize = 5;
    let fp: f64 = 0.01;
    let bloom = create_bloom(values, size, fp);

    let bloom_ron = ron::to_string(&bloom).unwrap();
    let deserialized: Bloom<String> = ron::from_str(&bloom_ron).unwrap();

    assert_eq!(
        bloom.check(&"test2".to_string()),
        deserialized.check(&"test2".to_string())
    );
    assert_eq!(
        bloom.check(&"test4".to_string()),
        deserialized.check(&"test4".to_string())
    );
}
