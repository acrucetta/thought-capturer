mod thought;

use chrono::prelude::*;
use clap::{arg, command, Command};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};

fn get_next_id() -> u32 {
    let file = File::open("thoughts.csv");

    // We want to return 1 if the file doesn't exist,
    // otherwise we want to return the next ID
    // To get the next ID we search the last line of the file in the ID column
    // and increment it by 1
    match file {
        Ok(f) => {
            let reader = BufReader::new(f);
            let mut last_line = String::new();
            for line in reader.lines() {
                last_line = line.unwrap();
            }
            let split: Vec<&str> = last_line.split(",").collect();
            let id = split[0].parse::<u32>().unwrap();
            id + 1
        }
        Err(_) => 1,
    }
}

fn get_current_timestamp() -> String {
    let utc: DateTime<Utc> = Utc::now();
    // Format the timestamp as YYYY-MM-DD
    utc.format("%Y-%m-%d").to_string()
}

fn add_thought(thought: &String) {
    // Prompt the user for tags (optional)
    let tags = {
        println!("Enter tags (optional):");
        let mut tags = String::new();
        std::io::stdin().read_line(&mut tags).unwrap();
        tags.trim().to_string()
    };

    // Generate a new ID and timestamp
    let id = get_next_id();
    let timestamp = get_current_timestamp();
    let message = thought.trim().to_string();

    // Create a new thought and append it to the CSV file
    let thought = thought::Thought {
        id,
        timestamp,
        message,
        tags,
    };

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("thoughts.csv")
        .unwrap();
    if file.metadata().unwrap().len() == 0 {
        writeln!(file, "id,timestamp,message,tags").unwrap();
    }
    writeln!(
        file,
        "{},{},{},{}",
        thought.id, thought.timestamp, thought.message, thought.tags
    )
    .unwrap();
}

fn list_thoughts() {
    if !std::path::Path::new("thoughts.csv").exists() {
        println!("No thoughts found, add one with `thg add (thought)`");
        return;
    } 
    let mut reader = csv::Reader::from_path("thoughts.csv").unwrap();
    for result in reader.deserialize::<thought::Thought>() {
        let thought = result.unwrap();
        println!(
            "{} {} {} ({})",
            thought.id, thought.timestamp, thought.message, thought.tags
        );
    }
}

fn remove_thought(id: &u32) {
    let file = File::open("thoughts.csv").unwrap();
    let reader = BufReader::new(file);
    let writer_file = File::create("temp.csv").unwrap();
    let mut writer = csv::Writer::from_writer(BufWriter::new(writer_file));

    for line in reader.lines() {
        let line = line.unwrap();
        let split: Vec<&str> = line.split(",").collect();
        let current_id = split[0].parse::<u32>().unwrap();
        if current_id != *id {
            writer.write_record(split).unwrap();
        }
    }
    std::fs::rename("temp.csv", "thoughts.csv").unwrap();
}

fn main() {
    let matches = command!()
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("add")
                .about("Add a new thought")
                .arg(arg!([THOUGHT]))
                .arg_required_else_help(true),
        )
        .subcommand(Command::new("ls").about("List all thoughts"))
        .subcommand(
            Command::new("rm")
                .about("Remove a thought")
                .arg(arg!([id]))
                .arg_required_else_help(true),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("add", sub_matches)) => {
            let thought = sub_matches.get_one::<String>("THOUGHT").unwrap();
            add_thought(thought);
        }
        Some(("ls", _sub_matches)) => list_thoughts(),
        Some(("rm", sub_matches)) => {
            let id = sub_matches.get_one::<u32>("id").unwrap();
            remove_thought(id);
        }
        _ => println!("No subcommand was used"),
    }
}
