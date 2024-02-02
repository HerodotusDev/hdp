use evaluator::aggregation_functions::integer::{
    average, count_if, find_max, find_min, standard_deviation,
};
use types::datalake::base::DataPoint;

#[test]
fn test_avg() {
    let values = vec![DataPoint::Int(1), DataPoint::Int(2), DataPoint::Int(3)];
    assert_eq!(average(&values).unwrap(), DataPoint::Float(2.0));

    let values = vec![DataPoint::Int(1), DataPoint::Int(2)];
    assert_eq!(average(&values).unwrap(), DataPoint::Float(1.5));
}

#[test]
fn test_avg_empty() {
    let values = vec![];
    assert!(average(&values).is_err());
}

#[test]
fn test_avg_mixed() {
    let values = vec![DataPoint::Int(1), DataPoint::Float(2.0)];
    assert_eq!(average(&values).unwrap(), DataPoint::Float(1.5));
}

#[test]
fn test_find_max() {
    let values = vec![DataPoint::Int(1), DataPoint::Int(2), DataPoint::Int(3)];
    assert_eq!(find_max(&values).unwrap(), DataPoint::Float(3.0));

    let values = vec![DataPoint::Int(1), DataPoint::Int(2)];
    assert_eq!(find_max(&values).unwrap(), DataPoint::Float(2.0));
}

#[test]
fn test_find_min() {
    let values = vec![DataPoint::Int(1), DataPoint::Int(2), DataPoint::Int(3)];
    assert_eq!(find_min(&values).unwrap(), DataPoint::Float(1.0));

    let values = vec![DataPoint::Int(1), DataPoint::Int(2)];
    assert_eq!(find_min(&values).unwrap(), DataPoint::Float(1.0));
}

#[test]
fn test_std() {
    let values = vec![DataPoint::Int(1), DataPoint::Int(2), DataPoint::Int(3)];
    assert_eq!(
        standard_deviation(&values).unwrap(),
        DataPoint::Float(0.816496580927726)
    );

    let values = vec![
        DataPoint::Int(0),
        DataPoint::Int(2),
        DataPoint::Int(10),
        DataPoint::Int(2),
        DataPoint::Int(100),
    ];
    assert_eq!(
        standard_deviation(&values).unwrap(),
        DataPoint::Float(38.75254830330516)
    );
}

#[test]
fn test_countif() {
    let values = vec![DataPoint::Int(1), DataPoint::Int(165), DataPoint::Int(3)];
    assert_eq!(count_if(&values, "04a5").unwrap(), DataPoint::Int(2));

    let values = vec![DataPoint::Int(1), DataPoint::Int(10)];
    assert_eq!(count_if(&values, "0000000000a").unwrap(), DataPoint::Int(1));
}
