use bloomfilter::Bloom;
use ocd_datalake_rs::{Datalake, DatalakeSetting};
use spinners::{Spinner, Spinners};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::time::Duration;
use std::{env, thread};

pub fn get_filename_from_path(path: &Path) -> Result<String, String> {
    match path.file_name().and_then(|name| name.to_str()) {
        Some(path) => Ok(path.to_string()),
        None => Err(format!("{}: No file found in path", path.display())),
    }
}

pub fn read_input_file(path: &PathBuf) -> Result<Vec<String>, io::Error> {
    let file: File = File::open(path)?;
    let reader: BufReader<File> = BufReader::new(file);
    let mut input: Vec<String> = Vec::new();
    for line in reader.lines() {
        input.push(line?.trim().to_string());
    }
    Ok(input)
}

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

pub fn create_bloom_from_queryhash(
    query_hash: String,
    environment: &String,
    positive_rate: f64,
) -> Result<Bloom<String>, String> {
    let dtl: Datalake = init_datalake(environment);
    let atom_values: Vec<String> = match fetch_atom_values_from_dtl(query_hash, dtl) {
        Ok(atom_values) => atom_values,
        Err(e) => return Err(e),
    };
    let size: usize = atom_values.len();

    let bloom: Bloom<String> = create_bloom(atom_values, size, positive_rate);
    Ok(bloom)
}

fn fetch_atom_values_from_dtl(
    query_hash: String,
    mut dtl: Datalake,
) -> Result<Vec<String>, String> {
    let mut sp = Spinner::new(Spinners::Line, "Waiting for data from Datalake...".into());

    let (sender, receiver) = mpsc::channel();
    let ui_thread = thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(500));
        match receiver.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => {
                thread::sleep(Duration::from_secs(1));
                break;
            }
            Err(TryRecvError::Empty) => {}
        }
    });

    let bulk_search_thread =
        thread::spawn(move || dtl.bulk_search(query_hash, vec!["atom_value".to_string()]));
    sender.send(()).unwrap();
    ui_thread.join().unwrap();
    let res = match bulk_search_thread.join().expect("Thread failed") {
        Ok(res) => {
            sp.stop_and_persist("✔", "Finished!".into());
            res
        }
        Err(e) => {
            sp.stop_and_persist("✗", "Failed.".into());
            return Err(format!("{}", e));
        }
    };
    let atom_values = dtl_csv_resp_to_vec(res);
    Ok(atom_values)
}

fn dtl_csv_resp_to_vec(csv: String) -> Vec<String> {
    let mut values: Vec<String> = Vec::new();
    for line in csv.lines() {
        if line.contains("atom_value") {
            continue;
        }
        values.push(line.trim().to_string())
    }
    values
}

fn init_datalake(environment: &String) -> Datalake {
    let username = get_username();
    let password = get_password();
    let dtl_setting = if environment == "preprod" {
        DatalakeSetting::preprod()
    } else {
        DatalakeSetting::prod()
    };
    Datalake::new(username, password, dtl_setting)
}

fn get_username() -> String {
    match env::var("OCD_DTL_RS_USERNAME") {
        Ok(username) => username,
        Err(_) => {
            println!("To avoid having to enter your username every time please set the environment variable OCD_DTL_RS_USERNAME.");
            println!("Please enter your username:");
            let mut username = String::new();
            io::stdin()
                .read_line(&mut username)
                .expect("Failed to read line");
            username.trim().to_string()
        }
    }
}

fn get_password() -> String {
    match env::var("OCD_DTL_RS_PASSWORD") {
        Ok(password) => password,
        Err(_) => {
            println!("To avoid having to enter your password every time, please set the environment variable OCD_DTL_RS_PASSWORD.");
            println!("Please enter your password:");
            let password = rpassword::read_password().unwrap();
            password.trim().to_string()
        }
    }
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

pub fn check_val_in_bloom(bloom: Bloom<String>, input: &Vec<String>) -> Vec<String> {
    let mut matches: Vec<String> = Vec::new();
    for value in input {
        if bloom.check(value) {
            matches.push(value.to_string());
        }
    }
    matches
}

#[test]
fn test_dtl_csv_resp_to_vec() {
    let csv = "test1\ntest2\ntest3\ntest4\ntest5\ntest6\ntest7\ntest8\ntest9\ntest10";
    let expected = vec![
        "test1".to_string(),
        "test2".to_string(),
        "test3".to_string(),
        "test4".to_string(),
        "test5".to_string(),
        "test6".to_string(),
        "test7".to_string(),
        "test8".to_string(),
        "test9".to_string(),
        "test10".to_string(),
    ];
    let res = dtl_csv_resp_to_vec(csv.to_string());
    assert_eq!(res, expected)
}
