use bloomfilter::Bloom;
use datalake_hunter::serialize_bloom;
#[path = "common.rs"]
mod common;

#[test]
fn test_bloom_serialization() {
    let bloom = common::create_test_bloom();
    let serialized1 = serialize_bloom(&bloom).unwrap();
    let deserialized: Bloom<String> = ron::from_str(&serialized1).unwrap();
    let serialized2 = serialize_bloom(&deserialized).unwrap();
    assert_eq!(serialized1, serialized2);
}

#[test]
fn test_bloom_check_post_serialization() {
    let bloom = common::create_test_bloom();
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
