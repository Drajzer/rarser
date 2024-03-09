use regex::Regex;
use sysinfo::{System};
use sqlx::{postgres::PgPoolOptions, Postgres};
use std::{fs, path};
use std::env;
mod models;
use models::DBStruct;
#[tokio::main]

async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() <2{
        println!(r#"
    
        Usage: rarser [flags]
        Flags:
        -c/--country specify Database country
        -w/--word find specific word
        -m/--mails search for mails (0,1)
        -d/--domain speicfy domain
        -tl/--tld specify TLD
        -a/--all search all databases (0,1)
        
        "#);
    }


    //Define default values for flags
    let mut country = "global";
    let mut word = "10";
    let mut mails: &str = "1";
    let mut domain = "*";
    let mut tld = "*";
    let mut all = "0";

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
             }else if args[i] == "-m," || args[i] == "--mails"{
                if i+1 < args.len(){
                    mails = &args[i + 1];
                }
            } else if args[i] == "-d" || args[i] == "--domain" {
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
            } 
        }
    }



    let env = fs::read_to_string(".env").unwrap();
    let (key,databaseUrl) = env.split_once('=').unwrap();
    let pool = PgPoolOptions::new().max_connections(50).connect(&databaseUrl).await.unwrap();

    let s = System::new_all();
    let maxFileSize =  ((s.total_memory() as f64/1_073_741_824.0).round())/2.0;

    let results = sqlx::query_as!(
        DBStruct,
        "SELECT * FROM dbs"
    ).fetch_all(&pool).await;


}
