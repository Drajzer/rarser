#![allow(non_snake_case)]
use colored::Colorize;
use lazy_static::lazy_static;
use rayon::prelude::*;
use regex::Regex;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, PgPool};
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File, Metadata};
use std::io::{BufRead, BufReader};
use std::env;
use std::sync::Mutex;
mod models;
use crate::models::DBStruct;
use std::fs::OpenOptions;
use std::io::Write;
use std::io;
//Error handling
//save results in file
//support xlsx

//Global variable to calculate number of matches
lazy_static! {
    static ref MATCHES: Mutex<u32> = Mutex::new(0);
}
//Increment global variabl MATCHES
fn incrementMatches() {

    let mut counter = MATCHES.lock().unwrap();
    
    *counter += 1;
    
}

fn saveToFile(filePath: &str, line: &str) -> io::Result<()> {
    let mut file = OpenOptions::new().append(true).create(true).open(filePath)?;
    writeln!(file, "{}", line)?;
    Ok(())
}


//Reading path file
fn readFile(path: &str, domain:&str, tld:&str, word:&str) -> std::io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let emailRegex = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b").unwrap();
    
    for line in reader.lines() {
        let ln = line?;
        printEmails(&ln, &emailRegex,domain,tld,word,path);
    }
    
    Ok(())
}
//Match all possible cases from arguments
fn printEmails(line: &str, emailRegex: &Regex,domain:&str,tld:&str, wordToFInd:&str, source: &str) {
    for word in line.split_whitespace() {
        if emailRegex.is_match(word) {
            let details = findDetails(word);
            match (domain, tld, wordToFInd) {
                ("*", "*", "*") => {
                    println!("{} {}\n {} {}", "FOUND: ".green(), word.green(), "SOURCE: ".blue(),source.blue());
                    saveToFile("output.txt", word);
                    incrementMatches()
            }
                (d, "*", "*") if &d == &details[1] => {
                    println!("{} {}\n{} {}", "FOUND: ".green(), word.green(), "SOURCE: ".blue(),source.blue());
                    saveToFile("output.txt", word);
                    incrementMatches()
                }
                ("*", t, "*") if &t == &details[2] => {
                    println!("{} {}\n{} {}", "FOUND: ".green(), word.green(), "SOURCE: ".blue(),source.blue());
                    saveToFile("output.txt", word);
                    incrementMatches()
                }
                ("*", "*", w) if &w == &details[0] => {
                    println!("{} {}\n{} {}", "FOUND: ".green(), word.green(), "SOURCE: ".blue(),source.blue());
                    saveToFile("output.txt", word);
                    incrementMatches()
                }
                (d, t, "*") if &d == &details[1] && &t == &details[2] => {
                    println!("{} {}\n{} {}", "FOUND: ".green(), word.green(), "SOURCE: ".blue(),source.blue());
                    saveToFile("output.txt", word);
                    incrementMatches()
                }
                (d, "*", w) if &d == &details[1] && &w == &details[0] => {
                    println!("{} {}\n{} {}", "FOUND: ".green(), word.green(), "SOURCE: ".blue(),source.blue());
                    saveToFile("output.txt", word);
                    incrementMatches()
                }
                ("*", t, w) if &t == &details[2] && &w == &details[0] => {
                    println!("{} {}\n{} {}", "FOUND: ".green(), word.green(), "SOURCE: ".blue(),source.blue());
                    saveToFile("output.txt", word);
                    incrementMatches()
                }
                (d, t, w) if &d == &details[1] && &t == &details[2] && &w == &details[0] => {
                    println!("{} {}\n{} {}", "FOUND: ".green(), word.green(), "SOURCE: ".blue(),source.blue());
                    saveToFile("output.txt", word);
                    incrementMatches()
                }
                _ => {}
            }
        }
    }
}
//Extract username, domain and TLD from email and push it in vector
fn findDetails(mail:&str) -> Vec<&str>{

    let domain = mail.find("@");
    let indexD = domain.unwrap();
    let mut tmp = 0;
    for x in mail.chars(){
        if tmp > 8{
            if x == '.'{
                break;
            }
        }
        tmp +=1;
        
    }
    let mut details: Vec<&str> = Vec::new();
    if tmp <= mail.len(){

        let domain = &mail[indexD+1..tmp];
        let word =  &mail[0..indexD];
        let tld =  &mail[tmp..mail.len()];
        details.push(word);
        details.push(domain);
        details.push(tld);
    }
    details

}
//Connect with databse, reading .env
async fn conncetDatabase()-> Result<PgPool, Box<dyn Error>>{
    let env_content = match fs::read_to_string(".env") {
        Ok(content) => content,
        Err(e) => {
            println!("Error reading .env file: {}", e);
            return Err(Box::new(e));
        }
    };

    let databaseUrl = match env_content.lines().next() {
        Some(line) => {
            let (_, url) = line.split_once('=').expect("Invalid .env file");
            url.trim().to_string()
        }
        None => {
            println!("No database url found in .env file");
            return Err("No database URL found in .env file".into());
        }
    };

    let pool = PgPoolOptions::new()
        .max_connections(50)
        .connect(&databaseUrl)
        .await?;

    Ok(pool)
    
}


//Add new database in sources db
async fn addSource(country: &String, tags: &String, path: &String, pool: Pool<Postgres>)-> Result<(), sqlx::Error>{

    let metaData:Metadata = match fs::metadata(path) {
        Ok(metadata) => {
            metadata
        }
        Err(e) => {
            println!("{}", e);
            return Err(e.into());
        }
        
    };
    let fileSize =  ((metaData.len() as f64/1_073_741_824.0).round())/2.0;

    sqlx::query!(
        "INSERT INTO sources (country, tags, path,size) VALUES ($1, $2, $3, $4)",
        country,
        tags,
        path,
        fileSize as i32,
    )
    .execute(&pool)
    .await?;

    Ok(())

}


#[tokio::main]
async fn main() -> std::io::Result<()>  {
    let args: Vec<String> = env::args().collect();



    if args.len() <2{
        println!(r#"
    
        Usage: rarser [flags]
        Flags:
        -c/--country specify Database country
        -w/--word find specific word in email  
        -d/--domain speicfy domain
        -tl/--tld specify TLD
        -t/--tags add tags
        -a/--all search all sources
        --add add database to sources [country] [tags] [path]
        
        "#);
        return Ok(());
    }
    
    let pool = conncetDatabase().await.expect("database error");


    if args[1] == "-add"{
        let countryDb = &args[2];
        let tagsDb: &String = &args[3];
        let pathDb = &args[4];
        match addSource(countryDb, tagsDb, pathDb, pool.clone()).await{
            Ok(()) => println!("Success"),
            Err(e)=>{
                println!("{} {}","Error: ".red(), e);
            }

        };
        return Ok(());
    }
    
    //Define default values for flags

    let mut country = "global";
    let mut word = "*";
    let mut domain = "*";
    let mut tld = "*";
    let mut all = "0";
    let mut tags = "unique";
    
    if args.len() > 1 {
        for i in 1..args.len() {
            if args[i] == "-c" || args[i] == "--country" {
                if i + 1 < args.len() {
                    country = &args[i + 1];
                }
            } else if args[i] == "-w" || args[i] == "--word" {
                if i + 1 < args.len() {
                    word = &args[i + 1];
                }
             }else if args[i] == "-d" || args[i] == "--domain" {
                if i + 1 < args.len() {
                    domain = &args[i + 1];
                }
            } else if args[i] == "-tl" || args[i] == "--tld" {
                if i + 1 < args.len() {
                    tld = &args[i + 1];
                }
            } else if args[i] == "-a" || args[i] == "--all"{
                if i+1 < args.len(){
                    all = &args[i + 1];
                }
            } else if args[i] == "-t" || args[i] == "--tags"{
                if i+1 < args.len(){
                    tags = &args[i + 1];
                }
            }
        }
    }
    println!(r#"
    
Your config:
    Country: {},
    Word: {},
    Domain: {},
    TLD: {},
    All: {},
    Tags: {}"#,country,word,domain,tld,all,tags);

    //Extract infos from sources DB
    let results = sqlx::query_as!(
        DBStruct,
        "SELECT * FROM sources"
    ).fetch_all(&pool).await;

    let tagovi: Vec<&str> = tags.split(",").collect();
    let mut priorityVec: Vec<HashMap<&str, usize>> = Vec::new();

    let binding = results.unwrap();
    for x in &binding{
        let mut tagCounts: HashMap<&str, usize> = HashMap::new();
        if all == "1"{
            let tagoviDb: Vec<&str> = x.tags.split(",").collect();
            for element in &tagoviDb {
                if tagovi.contains(&element) {
                    *tagCounts.entry(&x.path).or_insert(0) +=1;
                }
            }
            priorityVec.push(tagCounts);

        }
        else if  x.country == country{
            let tagoviDb: Vec<&str> = x.tags.split(",").collect();
            for element in &tagoviDb {
                if tagovi.contains(&element) {
                    *tagCounts.entry(&x.path).or_insert(0) +=1;
                }
            }
            priorityVec.push(tagCounts);
            
        }
    }
    //Makes priorty vector by counting number of matched tags
    priorityVec.sort_by(|a,b|{
        let countA: usize = a.values().sum();
        let countB: usize = b.values().sum();
        countB.cmp(&countA)
    });
    priorityVec.retain(|map| !map.is_empty());
    
    let mut paths: Vec<&str> = Vec::new();
    for x in priorityVec{
        for (path,_) in x{ 
            paths.push(path);
            
        }
    }

    // Implement rayon (threadpool)
    paths.par_iter().for_each(|&path| {
        
        readFile(path,domain,tld,word);
    });
    let counter = MATCHES.lock().unwrap();
    println!("{} {}", "Total matches:".red() ,*counter);


    Ok(())
}
