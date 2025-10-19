use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let query = if args.len() > 1 {
        args[1..].join(" ")
    } else {
        "rust tutorial".to_string()
    };

    let results = agpt_we::search(&query);

    if let Err(e) = agpt_we::save_results_markdown(&query, &results, "results") {
        eprintln!("Failed to save markdown: {}", e);
    }
}

