#[tokio::main]
async fn main() -> Result<(), codepecker::error::CodepeckerError> {
    if let Err(e) = codepecker::builder().await {
        eprintln!("程序运行出错: {}", e);
    }
    Ok(())
}
