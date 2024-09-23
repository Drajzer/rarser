#![allow(non_snake_case)]
use colored::Colorize;
use lazy_static::lazy_static;
use regex::Regex;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::sync::Mutex;
use std::fs::OpenOptions;
use std::io::Write;
use std::io;
use office::{Excel, DataType};
use sqlx::mysql::MySqlPool;
use sqlx::Error as SqlxError;
use std::io::Error as IoError;
use std::collections::HashMap;

mod models;
use crate::rarser::models::DBStruct;

//Global variable to calculate number of matches
lazy_static! {
    pub static ref MATCHES: Mutex<u32> = Mutex::new(0);
}
//Increment global variabl MATCHES
pub fn incrementMatches() {

    let mut counter = MATCHES.lock().unwrap();
    
    *counter += 1;
    
}

// Sort vectory by elements priority and extract paths
pub fn makePriorityVectory(mut priorityVector: Vec<HashMap<String, usize>>)-> Vec<String> {
    priorityVector.sort_by(|a, b| {
        let count_a: usize = a.values().sum();
        let count_b: usize = b.values().sum();
        count_b.cmp(&count_a)
    });
    priorityVector.retain(|map| !map.is_empty());

    let mut paths: Vec<String> = Vec::new();
    for x in &priorityVector {
        for (path, _) in x {
            paths.push(path.to_string());
        }
    }
    paths
}

// Fetch all sources from database
pub async fn fetchSources(pool: &MySqlPool) -> Result<Vec<DBStruct>, sqlx::Error> {
    let results = sqlx::query_as!(
        DBStruct,
        "SELECT * FROM sources"
    )
    .fetch_all(pool)
    .await?;

    Ok(results)
}

//Save linme in file
pub fn saveToFile(filePath: &str, line: &str) -> io::Result<()> {
    let mut file = OpenOptions::new().append(true).create(true).open(filePath)?;
    writeln!(file, "{}", line)?;
    Ok(())
}


//Reading xlsx/xls/xlsm
pub fn readXlsx(path: &str, domain: &str, tld: &str, word: &str,boolOption:bool,saveOption: bool) -> Result<(), Box<dyn std::error::Error>> {
    let email_regex: Regex =
        Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b")?;

    let mut workbook = Excel::open(path)?;

    let sheetNames = workbook.sheet_names()?;

    for name in sheetNames {
        if let Ok(range) = workbook.worksheet_range(&name) {
            for row in range.rows() {
                for cell in row {
                    if let DataType::String(s) = cell {
                        printEmails(&s, &email_regex, domain, tld, word, path,boolOption,saveOption);
                    }
                }
            }
        }
    }

    Ok(())
}

//Reading path file
pub fn readFile(path: &str, domain:&str, tld:&str, word:&str,printOption:bool,saveOption: bool) -> std::io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let emailRegex: Regex = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b").unwrap();
    
    for line in reader.lines() {
        let ln = line?;
        printEmails(&ln, &emailRegex,domain,tld,word,path, printOption,saveOption);
    }
    
    Ok(())
}
//Match all possible cases from arguments
pub fn printEmails(line: &str, emailRegex: &Regex, domain: &str, tld: &str, wordToFind: &str, source: &str, printOption:bool, saveOption: bool) {
    for word in line.split_whitespace() {
        if emailRegex.is_match(word) {
            let details = findDetails(word);
            let shouldPrint = match (domain, tld, wordToFind) {
                ("*", "*", "*") => true,
                (d, "*", "*") if d == details[1] => true,
                ("*", t, "*") if t == details[2] => true,
                ("*", "*", w) if w == details[0] => true,
                (d, t, "*") if d == details[1] && t == details[2] => true,
                (d, "*", w) if d == details[1] && w == details[0] => true,
                ("*", t, w) if t == details[2] && w == details[0] => true,
                (d, t, w) if d == details[1] && t == details[2] && w == details[0] => true,
                _ => false,
            };
            
            if shouldPrint {
                if printOption==true{
                    println!("{} {}\n{} {}", "FOUND: ".green(), word.green(), "SOURCE: ".blue(), source.blue());
                }
                else{
                    for email in emailRegex.find_iter(word){
                        println!("{}", email.as_str());
                    }
                }
                if saveOption == true{
                    let _ = saveToFile("output.txt",  emailRegex.find(word).expect("No email").into());

                }
                else {
                    
                    let _ = saveToFile("output.txt", word);
                }
                incrementMatches();
            }
        }
    }
}
//Extract username, domain and TLD from email and push it in vector
pub fn findDetails(mail: &str) -> Vec<&str> {
    let mut details: Vec<&str> = Vec::new();

    if let Some(index_at) = mail.find('@') {
        let username = &mail[0..index_at];

        let domainAndTld = &mail[index_at + 1..];

        if let Some(indexDot) = domainAndTld.find('.') {
            let domain = &domainAndTld[0..indexDot];
            let tld = &domainAndTld[indexDot + 1..];

            details.push(username);
            details.push(domain);
            details.push(tld);
        } else {
            details.push(username);
            details.push(domainAndTld);
            details.push(""); //NO TLD
        }
    } else {
        eprintln!("Invalid email format: {}", mail);
    }

    details
}
//Connect with databse, reading .env


pub async fn conncetDatabase() -> Result<MySqlPool, Box<dyn Error>> {
    // Read .env file content
    let env_content = match fs::read_to_string(".env") {
        Ok(content) => content,
        Err(e) => {
            println!("Error reading .env file: {}", e);
            return Err(Box::new(e));
        }
    };

    // Extract the database URL from .env file content
    let database_url = match env_content.lines().next() {
        Some(line) => {
            let (_, url) = line.split_once('=').expect("Invalid .env file format");
            url.trim().to_string()
        }
        None => {
            println!("No database URL found in .env file");
            return Err("No database URL found in .env file".into());
        }
    };

    // Create a connection pool for MySQL
    let pool = MySqlPool::connect(&database_url).await?;

    Ok(pool)
}
#[derive(Debug)]
pub enum AddSourceError {
    Io(()),
    Sqlx(()),
}

// Implement conversion from IoError to AddSourceError
impl From<IoError> for AddSourceError {
    fn from(error: IoError) -> Self {
        AddSourceError::Io(())
    }
}

// Implement conversion from SqlxError to AddSourceError
impl From<SqlxError> for AddSourceError {
    fn from(error: SqlxError) -> Self {
        AddSourceError::Sqlx(())
    }
}
// Add source files to Database
pub async fn addSource(country: &String, tags: &String, path: &String, pool: &MySqlPool) -> Result<(), AddSourceError> {

    let meta_data = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(e) => {
            println!("{}", e);
            return Err(e.into());
        }
    };

    let file_size = meta_data.len()as f64 / (1024.0 * 1024.0);
    sqlx::query!(
        "INSERT INTO sources (country, tags, path, size) VALUES (?, ?, ?, ?)",
        country,
        tags,
        path,
        file_size as i32
    )
    .execute(pool)
    .await
    .map_err(|e| e)?;

    Ok(())
}

