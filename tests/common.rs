#[cfg(test)]
use bloomfilter::Bloom;

pub fn create_test_bloom() -> Bloom<String> {
    let values: Vec<String> = vec![
        "test1".to_string(),
        "test2".to_string(),
        "test3".to_string(),
    ];
    let size: usize = 5;
    let fp: f64 = 0.01;
    datalake_hunter::create_bloom(values, size, fp)
}
