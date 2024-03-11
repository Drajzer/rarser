
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
use colored::Colorize;
//Error handling
//Add options for more countries, tlds, domains
//save results in file
fn readFile(path: &str, domain:&str) -> std::io::Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let emailRegex = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b").unwrap();
    
    for line in reader.lines() {
        let ln = line?;
        printEmails(&ln, &emailRegex,domain,path);
    }
    
    Ok(())
}
fn printEmails(line: &str, emailRegex: &Regex,domain:&str, source: &str) {
    let mut matchesSum = 0;
    if domain != "*"{
        for word in line.split_whitespace() {
            if emailRegex.is_match(word) {
                if findDetails(word)[1] == domain{
                    println!("{} {}\n{} {}","FOUND: ".green(),word.green(),"SOURCE: ".green(), source.green());
                    matchesSum+=1;
                }
            }
        }
    }
    else{
        for word in line.split_whitespace() {
            if emailRegex.is_match(word) {
                println!("{} {}","FOUND: ".green(),word.green());
                matchesSum+=1;

            }
        }
    }

}
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
        -t/--tags add tags
        
        "#);
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

    paths.par_iter().for_each(|&path| {
        
        readFile(path,domain);
    });


    Ok(())
}
