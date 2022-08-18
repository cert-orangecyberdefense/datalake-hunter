use bloomfilter::Bloom;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::path::PathBuf;

pub fn read_input_file(path: &PathBuf) -> Result<Vec<String>, io::Error> {
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

fn write_file(output_path: &PathBuf, content: String) -> Result<(), String> {
    let mut output_file = match File::create(&output_path) {
        Ok(output_file) => output_file,
        Err(e) => return Err(format!("{}: {}", output_path.display(), e)),
    };
    match write!(output_file, "{}", content) {
        Ok(()) => (),
        Err(e) => return Err(format!("{}: {}", output_path.display(), e)),
    }
    Ok(())
}

pub fn write_bloom_to_file(bloom: Bloom<String>, output_path: &PathBuf) -> Result<(), String> {
    let serialized_bloom = serialize_bloom(&bloom)?;
    write_file(output_path, serialized_bloom)
}

pub fn deserialize_bloom(path: &PathBuf) -> Result<Bloom<String>, String> {
    let ron_string = match std::fs::read_to_string(path) {
        Ok(ron_string) => ron_string,
        Err(e) => return Err(format!("{}: {}", path.display(), e)),
    };

    let bloom: Bloom<String> = match ron::from_str(&ron_string) {
        Ok(bloom) => bloom,
        Err(_) => {
            return Err(format!(
                "Failed to deserialize bloom filter located in {}",
                path.display()
            ))
        }
    };
    Ok(bloom)
}

pub fn serialize_bloom(bloom: &Bloom<String>) -> Result<String, String> {
    let serialized = ron::to_string(&bloom).expect("Failed to serialize the bloomfilter");
    Ok(serialized)
}

fn create_bloom(input: Vec<String>, size: usize, positive_rate: f64) -> Bloom<String> {
    let mut bloom: Bloom<String> = Bloom::new_for_fp_rate(size, positive_rate);
    for value in input {
        bloom.set(&value);
    }
    bloom
}

pub fn create_bloom_from_file(
    input_path: &PathBuf,
    positive_rate: f64,
) -> Result<Bloom<String>, String> {
    let input = match read_input_file(input_path) {
        Ok(input) => input,
        Err(e) => return Err(format!("{}: {}", input_path.display(), e)),
    };
    let size = input.len();

    let bloom = create_bloom(input, size, positive_rate);
    Ok(bloom)
}

pub fn create_bloom_from_queryhash() {}

pub fn check_val_bloom(bloom: Bloom<String>, input: Vec<String>) {}
