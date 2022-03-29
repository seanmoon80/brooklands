pub mod models;
pub mod constant;

use reqwest;
use md5;
use std::fs::File;
use std::io::Read;
use std::io::prelude::*;
use std::path::Path;
use std::error::Error;
use reqwest::Response;
use scraper::{Html, Selector};
use scraper::element_ref::ElementRef;
use chrono::{NaiveTime, Timelike};

use crate::models::entry::Entry;
use crate::constant::{BASE_URL};

#[derive(Debug)]
pub struct GrandPrixPage {
    season: i32,
    name: String,
    page_url: String
}

#[derive(Debug)]
pub struct GrandPrixSession {
    name: String,
    page_url: String,
}

#[derive(Debug)]
pub struct Driver {
    car_number: i32,
    name: String,
    nationality: String
}

#[derive(Debug)]
pub struct SessionData<'a> {
    session: &'a GrandPrixSession,
    driver: Driver,
    team: String,
    laps: i32,
    time: u32,
    gap: Option<String>,
    interval: Option<String>,
    best_lap_time: u32,
    best_lap_time_lap: u32
}

pub async fn get_page(page_url: &str) -> Result<Response, reqwest::Error> {
    reqwest::get(page_url).await
}

fn get_team_entries(entries: &mut Vec<Entry>, element: &ElementRef) {
    let row_selector = Selector::parse("tr").unwrap();
    let rows = element.select(&row_selector);
    let mut team = "";
    for row in rows {
        let mut cell_iter = row.children();
        team = ElementRef::wrap(cell_iter.next().unwrap()).unwrap().text().next().unwrap_or(team);
        let entry = Entry {
            season: None,
            team: String::from(team),
            car_number: ElementRef::wrap(cell_iter.next().unwrap()).unwrap().text().next().unwrap().parse::<i32>().unwrap(),
            driver_name: String::from(ElementRef::wrap(cell_iter.next().unwrap()).unwrap().text().next().unwrap()),
        };
        entries.push(entry);
    }
}

pub async fn get_page_content(page_url: &str) -> Result<String, std::io::Error> {
    let enc_url = md5::compute(page_url);
    let path = format!("./webpages/{:?}.html", enc_url);
    if !Path::new(&path).exists() {
        let page = get_page(page_url).await.unwrap().text().await.unwrap();
        let mut f = File::create(&path)?;
        f.write_all(page.as_bytes())?;
    }

    let mut file = File::open(&path)?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

pub async fn get_season_document(season: i32) -> Html {
    let page_url = format!("{}/series/formula-one/season/{}", BASE_URL, season);
    let page= get_page_content(&page_url).await.unwrap();
    Html::parse_document(&page)
}

pub async fn get_result_page(grandprix: &GrandPrixPage) -> Html {
    let page_url = format!("{}{}", BASE_URL, grandprix.page_url);
    let page = get_page_content(&page_url).await.unwrap();
    Html::parse_document(&page)
}

pub async fn get_entry_list(season: i32, document: &Html) {
    let mut entries:Vec<Entry> = Vec::new();
    let entry_table_selector = Selector::parse("table._2Q90P").unwrap();
    let entry_row_selector = Selector::parse("tbody._2xhp6").unwrap();
    let mut tables = document.select(&entry_table_selector);

    let entry_table = tables.next().unwrap();
    let entries_iter = entry_table.select(&entry_row_selector);
    for entry in entries_iter {
        get_team_entries(&mut entries, &entry);
    }
    models::insert_entries(&season, &entries).await.unwrap();
}

pub async fn get_grandprix_list(season: i32, document: &Html) -> Vec<GrandPrixPage> {
    let table_selector = Selector::parse("table._2Q90P").unwrap();
    let table_row_selector = Selector::parse("tbody._2xhp6 tr").unwrap();
    let mut tables = document.select(&table_selector);
    let calendar_elm = tables.nth(2).unwrap();

    let rows = calendar_elm.select(&table_row_selector);
    let page_list = rows
        .map(|row| {
            row.children().nth(2)
                .and_then(|cell| cell.children().nth(0))
                .and_then(|cell| ElementRef::wrap(cell))
                .and_then(|element| {
                    Some(GrandPrixPage {
                        season: season,
                        name: String::from(element.text().next().unwrap()),
                        page_url: String::from(element.value().attr("href").unwrap())
                    })
                }).unwrap()
        })
        .collect::<Vec<_>>();
    page_list
}

pub async fn get_grandprix_data(grandprix: &GrandPrixPage) {
    let document = get_result_page(&grandprix).await;
    let grandprix = get_grandprix_sessions(&document).await;
    get_classification_data(&grandprix[0], &document).await;
}

pub async fn get_grandprix_sessions(document: &Html) -> Vec<GrandPrixSession> {
    let session_tab_selector = Selector::parse("div._1CDKX").unwrap();
    let session_tab = document.select(&session_tab_selector).nth(1).unwrap();
    let sessions = session_tab
        .children()
        .map(|session| {
            ElementRef::wrap(session)
                .and_then(|element| {
                    Some(GrandPrixSession { 
                        name: element.inner_html(), 
                        page_url: String::from(element.value().attr("href").unwrap())
                    })
                })
                .unwrap()
        })
        .collect::<Vec<_>>();
    sessions
}

pub async fn get_classification_data(session: &GrandPrixSession, document: &Html) -> Result<(), Box<dyn Error>> {
    let row_selector = Selector::parse("tr._3AoAU").unwrap();
    let rows = document.select(&row_selector);
    let contents = rows.map(|row| {
        let row_contents = row
            .children()
            .into_iter()
            .map(|cell| {
                ElementRef::wrap(cell).unwrap().text().next()
            }).collect::<Vec<_>>();
        row_contents
    }).map(|row| {
        let driver = Driver {
            car_number: row[1].unwrap().parse::<i32>().unwrap(),
            name: String::from(row[2].unwrap()),
            nationality: String::from(row[3].unwrap())
        };
        let mut time_text = row[6].unwrap_or("");
        let time_format = String::from("%H:%M:%S%.3f");
        let time_separated: Vec<&str> = time_text.matches(":").collect();
        if time_separated.len() == 1 {
            time_text = format!("{}{}", "%H:", time_format)
        }
        (driver.name, time_text, time_format)
        //let total_time = NaiveTime::parse_from_str(time_text, &time_format);
        //total_time.unwrap_or(NaiveTime::from_hms(0,0,0)).num_seconds_from_midnight()
        /*
        SessionData {
            session: session,
            driver: driver,
            team: String::from(row[4].unwrap()),
            laps: row[5].unwrap().parse::<i32>().unwrap(),
        }
        */
    });
    for row in contents {
        println!("{:?}", row);
    }
    Ok(())
}

pub async fn laptime_to_seconds(time: &str) -> f32 {
    let time_and_seconds: Vec<&str> = time.split(".").collect();
    0.000
}