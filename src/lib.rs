use bloomfilter::Bloom;
use csv::{Reader, ReaderBuilder, Writer};
use ocd_datalake_rs::error::DatalakeError;
use ocd_datalake_rs::{Datalake, DatalakeSetting};
use spinners::{Spinner, Spinners};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::{self, prelude::*};
use std::path::{Path, PathBuf};

pub fn get_filename_from_path(path: &Path) -> Result<String, String> {
    match path.file_name().and_then(|name| name.to_str()) {
        Some(path) => Ok(path.to_string()),
        None => Err(format!("{}: No file found in path", path.display())),
    }
}

pub fn read_input_file(path: &PathBuf) -> Result<Vec<String>, io::Error> {
    let mut reader = ReaderBuilder::new().has_headers(false).from_path(path)?;
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

pub fn write_csv(
    matches: &HashMap<String, Vec<String>>,
    output: &PathBuf,
    no_header: &bool,
) -> Result<(), String> {
    let mut writer: Writer<File> = match Writer::from_path(output) {
        Ok(writer) => writer,
        Err(e) => return Err(format!("{}: {}", &output.display(), e)),
    };
    if !no_header {
        match writer.write_record(["matching_value", "bloom_filename"]) {
            // write the csv header
            Ok(()) => (),
            Err(e) => return Err(format!("{}: {}", &output.display(), e)),
        };
    }
    for (filename, values) in matches {
        for val in values {
            match writer.write_record([val, filename]) {
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

pub fn write_file(output_path: &PathBuf, content: String) -> Result<(), String> {
    let mut output_file: File = match File::create(output_path) {
        Ok(output_file) => output_file,
        Err(e) => return Err(format!("{}: {}", output_path.display(), e)),
    };
    match write!(output_file, "{}", content) {
        Ok(()) => (),
        Err(e) => return Err(format!("{}: {}", output_path.display(), e)),
    }
    Ok(())
}

pub fn write_bloom_to_file(bloom: &Bloom<String>, output_path: &PathBuf) -> Result<(), String> {
    let serialized_bloom: String = serialize_bloom(bloom)?;
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
    let mut spinner = Spinner::with_timer(Spinners::Line, "Creating bloom filter".to_string());
    let mut bloom: Bloom<String> = Bloom::new_for_fp_rate(size, positive_rate);
    for value in input {
        bloom.set(&value);
    }
    spinner.stop_and_persist("✔", "Finished creating the Bloom filter.".into());
    bloom
}

pub fn create_bloom_from_file(
    input_path: &PathBuf,
    positive_rate: f64,
) -> Result<Bloom<String>, String> {
    let mut spinner = Spinner::with_timer(Spinners::Line, "Reading input file...".to_string());
    let input: Vec<String> = match read_input_file(input_path) {
        Ok(input) => {
            spinner.stop_and_persist("✔", "Successfully extracted data from file.".into());
            input
        }
        Err(e) => {
            spinner.stop_and_persist("✗", "Failed.".into());
            return Err(format!("{}: {}", input_path.display(), e));
        }
    };
    let size: usize = input.len();
    if size == 0 {
        return Err(format!("{}: No data found in file", input_path.display()));
    }
    let bloom: Bloom<String> = create_bloom(input, size, positive_rate);
    Ok(bloom)
}

pub fn create_bloom_from_queryhash(
    query_hash: String,
    environment: &String,
    positive_rate: f64,
) -> Result<Bloom<String>, String> {
    let dtl: Datalake = match init_datalake(environment) {
        Ok(dtl) => dtl,
        Err(e) => return Err(format!("{}", e)),
    };
    let csv_string: String = fetch_atom_values_from_dtl(query_hash, dtl)?;
    let mut sp = Spinner::with_timer(Spinners::Line, "Extracting data...".into());
    let atom_values = match dtl_csv_resp_to_vec(csv_string) {
        Ok(atom_values) => {
            sp.stop_and_persist(
                "✔",
                "Successfully extracted data from Datalake response!".into(),
            );
            atom_values
        }
        Err(e) => {
            sp.stop_and_persist("✗", "Failed.".into());
            return Err(e);
        }
    };

    let size: usize = atom_values.len();
    if size == 0 {
        return Err("No data found in Datalake!".into());
    }
    let bloom: Bloom<String> = create_bloom(atom_values, size, positive_rate);
    Ok(bloom)
}

fn fetch_atom_values_from_dtl(query_hash: String, mut dtl: Datalake) -> Result<String, String> {
    let mut sp = Spinner::with_timer(
        Spinners::Line,
        format!("Waiting for data from Datalake for {}...", &query_hash),
    );

    let bulk_search_res = dtl.bulk_search(
        query_hash.clone(),
        vec![
            "atom_value".to_string(),
            ".hashes.md5".to_string(),
            ".hashes.sha1".to_string(),
            ".hashes.sha256".to_string(),
        ],
    );
    let atom_values = match bulk_search_res {
        Ok(atom_values) => {
            sp.stop_and_persist(
                "✔",
                format!(
                    "Successfully received data from Datalake for {}.",
                    &query_hash
                ),
            );
            atom_values
        }
        Err(e) => {
            sp.stop_and_persist("✗", "Failed.".into());
            match e {
                DatalakeError::ApiError(detailled_error) => {
                    let api_resp = match { detailled_error.api_response } {
                        Some(resp) => resp,
                        None => "API responded without a message.".to_string(),
                    };

                    return Err(format!("{} - {}", detailled_error.summary, api_resp));
                }
                _ => {
                    return Err(format!("{}", e));
                }
            }
        }
    };

    Ok(atom_values)
}

fn dtl_csv_resp_to_vec(csv: String) -> Result<Vec<String>, String> {
    let mut value_set: HashSet<String> = HashSet::new();

    let mut reader = Reader::from_reader(csv.as_bytes());
    for record in reader.records() {
        let record = record.unwrap();
        let (atom_value, hashes_md5, hashes_sha1, hashes_sha256): (String, String, String, String) =
            record.deserialize(None).unwrap();
        for hash in [atom_value, hashes_md5, hashes_sha1, hashes_sha256] {
            if !hash.is_empty() {
                value_set.insert(hash);
            }
        }
    }
    let atom_values = Vec::from_iter(value_set);

    Ok(atom_values)
}

fn init_datalake(environment: &String) -> Result<Datalake, io::Error> {
    let username = get_username()?;
    let password = get_password()?;
    let dtl_setting = if environment == "preprod" {
        DatalakeSetting::preprod()
    } else {
        DatalakeSetting::prod()
    };
    Ok(Datalake::new(username, password, dtl_setting))
}

fn get_username() -> Result<String, io::Error> {
    match env::var("OCD_DTL_RS_USERNAME") {
        Ok(username) => Ok(username),
        Err(_) => {
            println!("To avoid having to enter your username every time please set the environment variable OCD_DTL_RS_USERNAME.");
            println!("Please enter your username:");
            let mut username = String::new();
            match io::stdin().read_line(&mut username) {
                Ok(_) => (),
                Err(e) => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("{}", e),
                    ))
                }
            };
            Ok(username.trim().to_string())
        }
    }
}

fn get_password() -> Result<String, io::Error> {
    match env::var("OCD_DTL_RS_PASSWORD") {
        Ok(password) => Ok(password),
        Err(_) => {
            println!("To avoid having to enter your password every time, please set the environment variable OCD_DTL_RS_PASSWORD.");
            println!("Please enter your password:");
            let password = match rpassword::read_password() {
                Ok(password) => password,
                Err(e) => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("{}", e),
                    ))
                }
            };
            Ok(password.trim().to_string())
        }
    }
}

pub fn get_bloom_from_paths(
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

pub fn get_bloom_from_queryhashes(
    queryhashes: &Vec<String>,
    environment: &String,
    rate: f64,
) -> Result<HashMap<String, Bloom<String>>, String> {
    let mut blooms: HashMap<String, Bloom<String>> = HashMap::new();
    for queryhash in queryhashes {
        let bloom = create_bloom_from_queryhash(queryhash.to_string(), environment, rate)?;
        blooms.insert(queryhash.to_string(), bloom);
    }
    Ok(blooms)
}

pub fn check_val_in_bloom(bloom: Bloom<String>, input: &Vec<String>) -> Vec<String> {
    let mut matches: Vec<String> = Vec::new();
    for value in input {
        if bloom.check(value) {
            matches.push(value.to_string());
        }
    }
    matches
}

pub fn lookup_values_in_dtl(
    atom_values: Vec<String>,
    environment: &String,
) -> Result<String, String> {
    let mut dtl: Datalake = match init_datalake(environment) {
        Ok(dtl) => dtl,
        Err(e) => return Err(format!("{}", e)),
    };
    let mut sp = Spinner::with_timer(Spinners::Line, "Waiting for data from Datalake...".into());
    let csv_result: String = match dtl.bulk_lookup(atom_values) {
        Ok(csv_result) => {
            sp.stop_and_persist("✔", "Successfully fetched data from Datalake!".into());
            csv_result
        }
        Err(e) => {
            sp.stop_and_persist("✗", "Failed to fetch data from Datalake.".into());
            return Err(format!("{}", e));
        }
    };
    Ok(csv_result)
}

pub fn count_lookup_result_nb_lines(csv: &String) -> usize {
    let mut reader = Reader::from_reader(csv.as_bytes());
    let mut nb_lines = 0;
    for _ in reader.records() {
        nb_lines += 1;
    }
    nb_lines
}

#[test]
fn test_dtl_csv_resp_to_vec() {
    let csv_string: String = "atom_value,.hashes.md5,.hashes.sha1,.hashes.sha256\na50cb264d1979be3b3d766c0a7061372,a50cb264d1979be3b3d766c0a7061372,abe46855df32b6b46b71719e6d2d03c24285d1f4,b46e51a2e757f4d75f1a1fff1165c6a0503b687db6c7e672021dcaa9bedf2d88\n3005c03a7520a2db1f317c7551773355,3005c03a7520a2db1f317c7551773355,f7e5581cfb45c23d88951bd6afb47fc96fc7cd4b,\n188.227.106.122,,,\n1cdadad999b9e70c87560fcd9821c2b0fa4c0a92b8f79bded44935dd4fdc76a5,,,1cdadad999b9e70c87560fcd9821c2b0fa4c0a92b8f79bded44935dd4fdc76a5".to_string();
    let mut vec = match dtl_csv_resp_to_vec(csv_string) {
        Ok(vec) => vec,
        Err(e) => panic!("{}", e),
    };
    let mut expected = vec![
        "a50cb264d1979be3b3d766c0a7061372".to_string(),
        "abe46855df32b6b46b71719e6d2d03c24285d1f4".to_string(),
        "b46e51a2e757f4d75f1a1fff1165c6a0503b687db6c7e672021dcaa9bedf2d88".to_string(),
        "3005c03a7520a2db1f317c7551773355".to_string(),
        "f7e5581cfb45c23d88951bd6afb47fc96fc7cd4b".to_string(),
        "188.227.106.122".to_string(),
        "1cdadad999b9e70c87560fcd9821c2b0fa4c0a92b8f79bded44935dd4fdc76a5".to_string(),
    ];
    expected.sort();
    vec.sort();
    assert_eq!(vec, expected);
}
