use anyhow::Result;

pub async fn test_closer() -> Result<Vec<String>> {
    println!("test_closer");
    Ok(vec!["1".to_string()])
}
