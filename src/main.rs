
#[warn(non_snake_case)]
use regex::Regex;
use sqlx::PgPool;
use sysinfo::{System};
use sqlx::postgres::PgPoolOptions;
use std::fs;
use std::env;
use std::fs::read_to_string;
use std::path;
mod models;
use models::DBStruct;
use std::thread;
use std::time::Duration;
use std::collections::HashMap;
use std::fs::File;
use std::error::Error;
use std::io::{BufReader, BufRead};
use rayon::prelude::*;
//Error handling
//Multithreading
//Flags

fn readFile(path: &str) -> std::io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let ln = line?;
        println!("{}", ln);
    }

    Ok(())
}


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
        -a/--all search all databases (0,1)
        -t/--tags add tags
        
        "#);
    }


    //Define default values for flags
    let mut country = "global";
    let mut word = "10";
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
                    tld = &args[i + 1];
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


    let s = System::new_all();
    let maxFileSize =  ((s.total_memory() as f64/1_073_741_824.0).round())/2.0;

    let pool = conncetDatabase().await.expect("database error");

    let results = sqlx::query_as!(
        DBStruct,
        "SELECT * FROM sources"
    ).fetch_all(&pool).await;

    let tagovi: Vec<&str> = tags.split(",").collect();
    let mut priorityVec: Vec<HashMap<&str, usize>> = Vec::new();

    let binding = results.unwrap();
    for x in &binding{
        let mut tagCounts: HashMap<&str, usize> = HashMap::new();
        if x.size > maxFileSize as i32{
            println!("chunks");
        }
        if x.country == country{
            let tagoviDb: Vec<&str> = x.tags.split(",").collect();
            for element in &tagoviDb {
                if tagovi.contains(&element) {
                    *tagCounts.entry(&x.path).or_insert(0) +=1;
                }
            }
            priorityVec.push(tagCounts);
        }
        
    }
    priorityVec.sort_by(|a,b|{
        let countA: usize = a.values().sum();
        let countB: usize = b.values().sum();
        countB.cmp(&countA)
    });
    priorityVec.retain(|map| !map.is_empty());
    println!("{:?}",priorityVec);

    let mut paths: Vec<&str> = Vec::new();
    for x in priorityVec{
        for (path,_) in x{ 
            paths.push(path);

        }
    }
    paths.par_iter().for_each(|&path| {
        readFile(path);
    });

    let emailRegex = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b").unwrap();

    Ok(())
}
