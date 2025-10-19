use std::env;
use std::error::Error;
use agpt_we::web_engine::run_query;

// ------------------------------------
// Main function orchestrating everything
// ------------------------------------
fn main() -> Result<(), Box<dyn Error>> {
    // Read user query from CLI args
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run \"your query here\"");
        std::process::exit(1);
    }
    
    let query = args[1..].join(" ");
    let output_folder = "results";

    println!("[INFO] Starting web engine for query: {}", query);

    
    match run_query(&query, output_folder) {
        Ok(_) => println!("Done! Results saved successfully."),
        Err(e) => eprintln!("[ERROR] An error occurred: {}", e),
    }

    Ok(())
}

