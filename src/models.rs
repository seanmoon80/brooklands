use mysql_async::prelude::*;
use mysql_async::{Result};

pub mod entry;

pub async fn create_database() -> Result<()> {
    let pool = mysql_async::Pool::new("mysql://root:password@localhost:12000");
    let mut conn = pool.get_conn().await?;
    conn.query_drop(
        r"CREATE DATABASE IF NOT EXISTS brooklands"
    ).await?;

    conn.query_drop(
        r"USE brooklands"
    ).await?;

    conn.query_drop(
        r"CREATE TABLE IF NOT EXISTS entry_list (
            entry_id INTEGER PRIMARY KEY AUTO_INCREMENT,
            season YEAR NOT NULL,
            team VARCHAR(50) NOT NULL,
            car_number INTEGER NOT NULL,
            driver_name VARCHAR(100) NOT NULL
        )"
    ).await?;

    conn.query_drop(
        r"CREATE TABLE IF NOT EXISTS grandprix_list (
            grandprix_id INTEGER PRIMARY KEY AUTO_INCREMENT,
            season YEAR NOT NULL,
            name VARCHAR(100) NOT NULL,
            page_url VARCHAR(100) NOT NULL
        )"
    ).await?;

    conn;
    pool.disconnect().await?;
    Ok(())
}

pub async fn insert_entries(season: &i32, entries: &Vec<entry::Entry>) -> Result<()> {
    let pool = mysql_async::Pool::new("mysql://root:password@localhost:12000/brooklands");
    let mut conn = pool.get_conn().await?;
    let params = entries.into_iter().map(|entry| {
        params! {
            "season" => season,
            "team" => entry.team.as_str(),
            "car_number" => entry.car_number,
            "driver_name" => entry.driver_name.as_str()
        }
    });
    conn.exec_batch(
        r"INSERT INTO `entry_list` (season, team, car_number, driver_name)
               VALUES (:season, :team, :car_number, :driver_name)",
            params
    ).await?;

    conn;
    pool.disconnect().await?;
    Ok(())
}