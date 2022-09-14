use bloomfilter::Bloom;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::path::{Path, PathBuf};

pub fn get_filename_from_path(path: &Path) -> Result<String, String> {
    match path.file_name().and_then(|name| name.to_str()) {
        Some(path) => Ok(path.to_string()),
        None => Err(format!("{}: No file found in path", path.display())),
    }
}

pub fn read_input_file(path: &PathBuf) -> Result<Vec<String>, io::Error> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)?;
    let mut input: Vec<String> = Vec::new();
    for result in reader.records() {
        let record = result?;
        let atom: String = match record.get(0) {
            Some(atom) => atom.trim().to_string(),
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("{}: No data found in file", path.display()),
                ))
            }
        };
        input.push(atom);
    }
    Ok(input)
}

// pub fn read_input_file(path: &PathBuf) -> Result<Vec<String>, io::Error> {
//     let file: File = File::open(path)?;
//     let reader: BufReader<File> = BufReader::new(file);
//     let mut input: Vec<String> = Vec::new();
//     for line in reader.lines() {
//         input.push(line?);
//     }
//     Ok(input)
// }

pub fn write_csv(
    matches: &HashMap<String, Vec<String>>,
    output: &PathBuf,
    no_header: &bool,
) -> Result<(), String> {
    let mut writer: csv::Writer<File> = match csv::Writer::from_path(&output) {
        Ok(writer) => writer,
        Err(e) => return Err(format!("{}: {}", &output.display(), e)),
    };
    if !no_header {
        match writer.write_record(&["matching_value", "bloom_filename"]) {
            // write the csv header
            Ok(()) => (),
            Err(e) => return Err(format!("{}: {}", &output.display(), e)),
        };
    }
    for (filename, values) in matches {
        for val in values {
            match writer.write_record(&[val, filename]) {
                Ok(()) => (),
                Err(e) => return Err(format!("{}: {}", &output.display(), e)),
            }
        }
    }
    match writer.flush() {
        // flush the internal buffer
        Ok(()) => (),
        Err(e) => return Err(format!("{}: {}", &output.display(), e)),
    };
    Ok(())
}

fn write_file(output_path: &PathBuf, content: String) -> Result<(), String> {
    let mut output_file: File = match File::create(&output_path) {
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
    let serialized_bloom: String = serialize_bloom(&bloom)?;
    write_file(output_path, serialized_bloom)
}

pub fn deserialize_bloom(path: &PathBuf) -> Result<Bloom<String>, String> {
    let ron_string: String = match std::fs::read_to_string(path) {
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
    let serialized: String = ron::to_string(&bloom).expect("Failed to serialize the bloomfilter");
    Ok(serialized)
}

pub fn create_bloom(input: Vec<String>, size: usize, positive_rate: f64) -> Bloom<String> {
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
    let input: Vec<String> = match read_input_file(input_path) {
        Ok(input) => input,
        Err(e) => return Err(format!("{}: {}", input_path.display(), e)),
    };
    let size: usize = input.len();

    let bloom: Bloom<String> = create_bloom(input, size, positive_rate);
    Ok(bloom)
}

pub fn get_bloom_from_path(
    bloom_paths: &Vec<PathBuf>,
) -> Result<HashMap<String, Bloom<String>>, String> {
    let mut blooms: HashMap<String, Bloom<String>> = HashMap::new();
    for path in bloom_paths {
        let filename = get_filename_from_path(path)?;
        let bloom = deserialize_bloom(path)?;
        blooms.insert(filename, bloom);
    }
    Ok(blooms)
}

pub fn create_bloom_from_queryhash() {}

pub fn check_val_in_bloom(bloom: Bloom<String>, input: &Vec<String>) -> Vec<String> {
    let mut matches: Vec<String> = Vec::new();
    for value in input {
        if bloom.check(value) {
            matches.push(value.to_string());
        }
    }
    matches
}

pub fn lookup_values_in_dtl(atom_values: Vec<String>, output: &PathBuf) -> Result<String, String> {
    Ok("".to_string())
}
