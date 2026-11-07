use std::env;
use std::io::{self, Read};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let query = if !args.is_empty() {
        args.join(" ")
    } else {
        let mut buffer = String::new();
        let _ = io::stdin().read_to_string(&mut buffer);
        buffer.trim().to_string()
    };

    let filters = species_search::parse_query(&query);
    match serde_json::to_string(&filters) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("Error serializing filters: {}", e);
            std::process::exit(1);
        }
    }
}
