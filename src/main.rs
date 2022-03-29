use tokio;
use std::error::Error;
use std::io;
use brooklands_api::{get_entry_list, get_season_document, get_grandprix_list, get_grandprix_data, get_page};
use brooklands_api::models::create_database;
use chrono::NaiveTime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let season = 2021;
    create_database().await?;
    let document = get_season_document(season).await;
    let grandprix = get_grandprix_list(season, &document).await;
    get_grandprix_data(&grandprix[0]).await;
    Ok(())
}
