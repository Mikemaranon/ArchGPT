use std::error::Error;
use std::fs;
use std::path::Path;
use headless_chrome::{Browser, LaunchOptions};
use urlencoding::decode;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub url: String,
    pub title: String,
}

// -----------------------------
// Search for URLs for user query  
// -----------------------------
pub fn search(query: &str) -> Vec<SearchResult> {
    println!("Searching in DuckDuckGo: {}", query);

    // Build DuckDuckGo search URL
    let url = format!("https://duckduckgo.com/html/?q={}", query.replace(" ", "+"));

    // Launch headless Chrome silently
    let browser = Browser::new(LaunchOptions {
        headless: true,
        ..Default::default()
    }).expect("Failed to launch headless Chrome");

    let tab = browser.new_tab().expect("Failed to create tab");

    // Navigate to DuckDuckGo search page
    tab.navigate_to(&url).expect("Failed to navigate");
    tab.wait_until_navigated().expect("Navigation failed");

    // Get page HTML
    let html = tab.get_content().expect("Failed to get page content");

    let mut results = Vec::new();

    // Extract URLs and titles manually
    for line in html.lines() {
        if line.contains("result__a") && line.contains("uddg=") {
            if let Some(start) = line.find("uddg=") {
                let encoded = &line[start + 5..];
                if let Some(end) = encoded.find('"') {
                    if let Ok(real_url) = decode(&encoded[..end]) {
                        // Extract title if possible
                        let title = if let Some(title_start) = line.find('>') {
                            let rest = &line[title_start+1..];
                            if let Some(title_end) = rest.find('<') {
                                rest[..title_end].to_string()
                            } else {
                                real_url.to_string()
                            }
                        } else {
                            real_url.to_string()
                        };

                        results.push(SearchResult {
                            url: real_url.to_string(),
                            title,
                        });
                    }
                }
            }
        }

        if results.len() >= 5 {
            break; // only top 5 results
        }
    }

    results
}

// -----------------------------
// Fetch page content (raw HTML)
// -----------------------------
pub fn fetch_content(url: &str) -> Result<String, Box<dyn Error>> {
    let browser = Browser::new(LaunchOptions {
        headless: true,
        ..Default::default()
    })?;

    let tab = browser.new_tab()?;
    tab.navigate_to(url)?;
    tab.wait_until_navigated()?;
    let html = tab.get_content()?; // Entire HTML
    Ok(html)
}

// -----------------------------
// Save results with full HTML
// -----------------------------
pub fn save_results_markdown(query: &str, results: &[SearchResult], folder: &str) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(folder)?;
    let path = Path::new(folder).join("results.md");

    let mut md = format!("# Query: {}\n## Results\n\n", query);

    // List results with titles and URLs
    for r in results.iter() {
        md.push_str(&format!("- [{}]({})\n", r.title, r.url));
    }

    md.push_str("\n### Content\n\n");

    // Fetch and append page content
    for r in results.iter() {
        md.push_str(&format!("#### {}\n", r.title));
        match fetch_content(&r.url) {
            Ok(html) => md.push_str(&format!("{}\n\n", html)),
            Err(e) => md.push_str(&format!("Failed to fetch content: {}\n\n", e)),
        }
    }

    fs::write(&path, md)?;
    println!("✅ Results saved to {}/results.md", folder);
    Ok(())
}

