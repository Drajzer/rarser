#![allow(non_snake_case)]
use rarser::makePriorityVectory;
use rayon::prelude::*;
use std::collections::HashMap;
use std::env;
use std::collections::HashSet;
mod rarser;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!(r#"
        Usage: rarser [flags]
        Flags:
        -c/--country specify Database country
        -w/--word find specific word in email  
        -d/--domain specify domain
        -tl/--tld specify TLD
        -t/--tags add tags
        -a/--all search all sources
        -l/--line print whole line
        -add add database to sources [country] [tags] [path]
        -m/--mail save only mails
        "#);
        return Ok(());
    }

    let pool = rarser::conncetDatabase().await?;

    if args[1] == "-add" {
        let countryDb = &args[2];
        let tagsDb = &args[3];
        let pathDb = &args[4];
        match rarser::addSource(countryDb, tagsDb, pathDb, &pool).await {
            Ok(()) => println!("Success"),
            Err(e) => eprintln!("Error: {:?}", e),
        }
        return Ok(());
    }

    // Define default values for flags
    let mut country = "global";
    let mut word = "*";
    let mut domain = "*";
    let mut tld = "*";
    let mut all = "0";
    let mut tags = "unique";
    let mut printOption:bool=false;
    let mut saveOption: bool = false;
    if args.contains(&"-l".to_string())|| args.contains(&"--line".to_string()){
        printOption=true;
    }
    if args.contains(&"-m".to_string())|| args.contains(&"--mail".to_string()){
        saveOption=true;
    }

    if args.len() > 1 {
        for i in 1..args.len() {
            match args[i].as_str() {
                "-c" | "--country" if i + 1 < args.len() => {
                    country = &args[i + 1];
                }
                "-w" | "--word" if i + 1 < args.len() => {
                    word = &args[i + 1];
                }
                "-d" | "--domain" if i + 1 < args.len() => {
                    domain = &args[i + 1];
                }
                "-tl" | "--tld" if i + 1 < args.len() => {
                    tld = &args[i + 1];
                }
                "-a" | "--all" if i + 1 < args.len() => {
                    all = &args[i + 1];
                }
                "-t" | "--tags" if i + 1 < args.len() => {
                    tags = &args[i + 1];
                }
                _ => {}
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
    Tags: {}
"#, country, word, domain, tld, all, tags);

    let tagovi: Vec<&str> = tags.split(',').collect();
    let mut priorityVector: Vec<HashMap<String, usize>> = Vec::new();

    let binding = rarser::fetchSources(&pool).await?;
    for item in binding.iter() {
        let dbTags: HashSet<&str> = item.tags.as_str().split(',').collect();
        let mut matchCount = 0;

        // Check country
        if country == "*" || item.country == country {
            matchCount += 1;
        }
        let tagoviSet: HashSet<&str> = tagovi.iter().copied().collect();
        // Check tags
        let uniqueMatch: HashSet<&str> = dbTags.intersection(&tagoviSet).copied().collect();
        if !tagovi.is_empty() && !uniqueMatch.is_empty() {
            matchCount += uniqueMatch.len();
        }
    
        // Add more checks here for word, domain, tld, all if needed

        // If there are matches, add the item to the priority vector with the match count
        if matchCount > 0 {
            let mut entry = HashMap::new();
            entry.insert(item.path.clone(), matchCount);
            priorityVector.push(entry);
        }
    }

    // Paths vectory by priority

    let mut paths: Vec<String> = makePriorityVectory(priorityVector);


    // Implement rayon (threadpool)
    paths.par_iter().for_each(|path| {
        let extension = match path.rfind('.') {
            Some(index) => &path[index + 1..],
            None => "",
        };
        if extension == "xlsx" || extension == "xls" || extension == "xlsm" {
            let _ = rarser::readXlsx(&path, domain, tld, word,printOption,saveOption);
        } else {
            let _ = rarser::readFile(&path, domain, tld, word,printOption,saveOption);
        }
    });

    let counter = rarser::MATCHES.lock().unwrap();
    println!("Total matches: {}", *counter);

    Ok(())
}