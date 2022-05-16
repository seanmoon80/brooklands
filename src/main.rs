use tokio;
use std::error::Error;
use std::io;
use brooklands_api::{
    get_entry_list, 
    get_season_document, 
    get_grandprix_list, 
    get_grandprix_data, 
    get_page,
    get_all_seasons
};
use brooklands_api::models::create_database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    create_database().await?;
    let seasons = get_all_seasons().await;
    //for season in seasons {
        let document = get_season_document(seasons[0]).await;
        //get_entry_list(seasons[0], &document).await;
        let grandprix = get_grandprix_list(seasons[0], &document).await;
        //for gp in grandprix {
            get_grandprix_data(seasons[0], &grandprix[3]).await;
        //}
    //}
    Ok(())
}
