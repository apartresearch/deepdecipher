use anyhow::Result;
use deepdecipher::server;

#[tokio::main]
pub async fn main() -> Result<()> {
    let api = server::api_doc();
    let api_json = serde_json::to_string_pretty(&api)?;
    std::fs::write("api.json", api_json)?;

    Ok(())
}
