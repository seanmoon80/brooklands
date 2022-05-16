use mysql_async::prelude::*;
use mysql_async::{Result};

use crate::GrandPrixPage;
use crate::SessionData;

pub mod entry;

pub async fn create_database() -> Result<()> {
    let pool = mysql_async::Pool::new("mysql://root:password@127.0.0.1:12000");
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
            driver_name VARCHAR(100) NOT NULL,
            UNIQUE KEY `entry_season_team_driver_uk` (`season`,`team`,`driver_name`)
        )"
    ).await?;

    conn.query_drop(
        r"CREATE TABLE IF NOT EXISTS grandprix_list (
            `grandprix_id` INTEGER PRIMARY KEY AUTO_INCREMENT,
            `season` YEAR NOT NULL,
            `name` VARCHAR(100) NOT NULL,
            `page_url` VARCHAR(100) NOT NULL,
            UNIQUE KEY `grandprix_season_name_uk` (`season`,`name`)
        )"
    ).await?;

    conn.query_drop(
        r"CREATE TABLE IF NOT EXISTS session_data (
            `session_data_id` INTEGER PRIMARY KEY AUTO_INCREMENT,
            `season` YEAR NOT NULL,
            `grandprix` VARCHAR(100) NOT NULL,
            `name` VARCHAR(100) NOT NULL,
            `car_number` INTEGER NOT NULL,
            `driver_name` VARCHAR(100) NOT NULL,
            `team` VARCHAR(50) NOT NULL,
            `laps` INTEGER NULL,
            `time` INTEGER NULL,
            `gap` VARCHAR(50) NULL,
            `interval` VARCHAR(10) NULL,
            `best_lap_time` INTEGER NULL,
            `best_lap_time_lap` INTEGER NULL,
            UNIQUE KEY `session_data_uk` (`season`,`car_number`,`name`)
        )"
    ).await?;

    conn;
    pool.disconnect().await?;
    Ok(())
}

pub async fn insert_entries(season: &u16, entries: &Vec<entry::Entry>) -> Result<()> {
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
        r"INSERT IGNORE INTO `entry_list` (season, team, car_number, driver_name)
               VALUES (:season, :team, :car_number, :driver_name)",
            params
    ).await?;

    conn;
    pool.disconnect().await?;
    Ok(())
}

pub async fn insert_grandprix(season: &u16, grandprix: &Vec<GrandPrixPage>) -> Result<()> {
    let pool = mysql_async::Pool::new("mysql://root:password@localhost:12000/brooklands");
    let mut conn = pool.get_conn().await?;
    let params = grandprix.into_iter().map(|gp| {
        params! {
            "season" => season,
            "name" => gp.name.as_str(),
            "page_url" => gp.page_url.as_str()
        }
    });
    conn.exec_batch(
        r"INSERT IGNORE INTO `grandprix_list` (season, name, page_url)
               VALUES (:season, :name, :page_url)",
               params
    ).await?;
    
    conn;
    pool.disconnect().await?;
    Ok(())
}

pub async fn insert_session_data<'a>(grandprix: &GrandPrixPage, season: &u16, session_data: &Vec<SessionData<'a>>) -> Result<()> {
    let pool = mysql_async::Pool::new("mysql://root:password@localhost:12000/brooklands");
    let mut conn = pool.get_conn().await?;
    let params = session_data.into_iter().map(|data| {
        params! {
            "season" => season,
            "grandprix" => &grandprix.name,
            "name" => &data.session.name,
            "car_number" => data.driver.car_number,
            "driver_name" => &data.driver.name,
            "team" => &data.team,
            "laps" => data.laps,
            "time" => data.time,
            "gap" => &data.gap,
            "interval" => &data.interval,
            "best_lap_time" => data.best_lap_time,
            "best_lap_time_lap" => data.best_lap_time_lap
        }
    });

    conn.exec_batch(
        r"INSERT IGNORE INTO `session_data` (`season`, `grandprix`, `name`, `car_number`, `driver_name`, `team`, `laps`, `time`, `gap`, `interval`, `best_lap_time`, `best_lap_time_lap`)
               VALUES (:season, :grandprix, :name, :car_number, :driver_name, :team, :laps, :time, :gap, :interval, :best_lap_time, :best_lap_time_lap)",
               params
    ).await?;

    conn;
    pool.disconnect().await?;
    Ok(())
}
