use evaluator::aggregation_functions::integer::{
    average, count_if, find_max, find_min, standard_deviation,
};

#[test]
fn test_avg() {
    let values = vec!["1".to_string(), "2".to_string(), "3".to_string()];
    assert_eq!(average(&values).unwrap(), "2000000000000000000".to_string());

    let values = vec!["1".to_string(), "2".to_string()];
    assert_eq!(average(&values).unwrap(), "1500000000000000000".to_string());
}

#[test]
fn test_avg_empty() {
    let values = vec![];
    assert!(average(&values).is_err());
}

#[test]
fn test_find_max() {
    let values = vec!["1".to_string(), "2".to_string(), "3".to_string()];
    assert_eq!(
        find_max(&values).unwrap(),
        "3000000000000000000".to_string()
    );

    let values = vec!["1".to_string(), "2".to_string()];
    assert_eq!(
        find_max(&values).unwrap(),
        "2000000000000000000".to_string()
    );
}

#[test]
fn test_find_min() {
    let values = vec!["1".to_string(), "2".to_string(), "3".to_string()];
    assert_eq!(
        find_min(&values).unwrap(),
        "1000000000000000000".to_string()
    );

    let values = vec!["1".to_string(), "2".to_string()];
    assert_eq!(
        find_min(&values).unwrap(),
        "1000000000000000000".to_string()
    );
}

#[test]
fn test_std() {
    let values = vec!["1".to_string(), "2".to_string(), "3".to_string()];
    assert_eq!(
        standard_deviation(&values).unwrap(),
        "816496580927726080".to_string()
    );

    let values = vec![
        "0".to_string(),
        "2".to_string(),
        "10".to_string(),
        "2".to_string(),
        "100".to_string(),
    ];
    assert_eq!(
        standard_deviation(&values).unwrap(),
        "38752548303305162752".to_string()
    );
}

#[test]
fn test_countif() {
    let values = vec!["1".to_string(), "165".to_string(), "3".to_string()];
    assert_eq!(
        count_if(&values, "04a5").unwrap(),
        "2000000000000000000".to_string()
    );

    let values = vec!["1".to_string(), "10".to_string()];
    assert_eq!(
        count_if(&values, "0000000000a").unwrap(),
        "1000000000000000000".to_string()
    );
}
