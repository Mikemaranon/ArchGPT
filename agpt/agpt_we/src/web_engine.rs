// --- helper imports at top of file ---
use headless_chrome::{Browser, LaunchOptionsBuilder, Tab};
use std::time::Duration;
use urlencoding::encode;

use std::fs;
use std::path::Path;
use std::error::Error;
use std::ffi::OsStr;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub content: Option<String>,
}

// ------------------------------------
// Robust navigation helper
// ------------------------------------

fn navigate_with_retries(tab: &Tab, url: &str, wait_selector: &str, retries: u8, wait_ms: u64) -> Result<(), Box<dyn std::error::Error>> {
    // Try several times to navigate and wait for a selector to appear
    for attempt in 1..=retries {
        if let Err(e) = tab.navigate_to(url) {
            eprintln!("[WARN] navigate_to failed (attempt {}): {}", attempt, e);
            std::thread::sleep(Duration::from_millis(wait_ms));
            continue;
        }

        // Try to wait until basic navigation event completes (non-blocking)
        let _ = tab.wait_until_navigated();

        // Now wait for the specific selector that signals the page is ready
        match tab.wait_for_element(wait_selector) {
            Ok(_) => {
                // small extra sleep to let JS finish rendering
                std::thread::sleep(Duration::from_millis(1000));
                return Ok(());
            }
            Err(e) => {
                eprintln!("[WARN] selector '{}' not found (attempt {}): {}", wait_selector, attempt, e);
                std::thread::sleep(Duration::from_millis(wait_ms));
                continue;
            }
        }
    }

    Err(format!("[ERROR] Navigation failed: selector '{}' never appeared after {} attempts", wait_selector, retries).into())
}

// ------------------------------------
// Search function (uses encode + navigate_with_retries)
// ------------------------------------

pub fn search(query: &str, tab: &Tab) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
    println!("Searching DuckDuckGo for: {}", query);

    // Properly encode query
    let encoded_q = encode(query);
    let url = format!("https://duckduckgo.com/html/?q={}", encoded_q);

    // Wait for anchor elements that contain results; use retries and short waits
    navigate_with_retries(tab, &url, "a.result__a", 5, 1000)?;

    let html = tab.get_content()?;
    let mut results = Vec::new();

    for line in html.lines() {
        if line.contains("result__a") && line.contains("uddg=") {
            if let Some(start) = line.find("uddg=") {
                let encoded = &line[start + 5..];
                if let Some(end) = encoded.find('"') {
                    if let Ok(real_url) = urlencoding::decode(&encoded[..end]) {
                        let title = if let Some(t1) = line.find('>') {
                            let rest = &line[t1 + 1..];
                            if let Some(t2) = rest.find('<') {
                                rest[..t2].to_string()
                            } else {
                                real_url.to_string()
                            }
                        } else {
                            real_url.to_string()
                        };

                        results.push(SearchResult {
                            title,
                            url: real_url.to_string(),
                            content: None,
                        });

                        if results.len() >= 5 {
                            break;
                        }
                    }
                }
            }
        }
    }

    println!("Found {} results", results.len());
    Ok(results)
}

// ------------------------------------
// Fetch content (uses navigate_with_retries for robustness)
// ------------------------------------

pub fn fetch_content(tab: &Tab, url: &str) -> Result<String, Box<dyn std::error::Error>> {
    println!("Fetching content from: {}", url);

    // Use body as a conservative selector for page readiness
    navigate_with_retries(tab, url, "body", 4, 1500)?;

    // Let dynamic content settle a bit more (some pages need longer)
    std::thread::sleep(Duration::from_secs(2));

    let html = tab.get_content()?;
    Ok(html)
}

// ------------------------------------
// Save all results as Markdown
// ------------------------------------

pub fn save_results_markdown(
    query: &str,
    results: &[SearchResult],
    folder: &str,
) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(folder)?;
    let path = Path::new(folder).join("results.md");

    let mut md = format!("# Query: {}\n\n## Results\n", query);

    for r in results {
        md.push_str(&format!("- [{}]({})\n", r.title, r.url));
    }

    md.push_str("\n---\n\n## Page Content\n");

    for r in results {
        md.push_str(&format!(
            "\n### {}\n{}\n",
            r.title,
            r.content.as_deref().unwrap_or("No content fetched.")
        ));
    }

    fs::write(&path, md)?;
    println!("📄 Results saved to {}/results.md", folder);
    Ok(())
}

// ------------------------------------
// run_query: launch browser with additional flags
// ------------------------------------

pub fn run_query(query: &str, output_folder: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Use LaunchOptionsBuilder to add args
    let launch_opts = LaunchOptionsBuilder::default()
        .headless(true)
        .args(vec![
            OsStr::new("--no-sandbox"),
            OsStr::new("--disable-gpu"),
            OsStr::new("--disable-dev-shm-usage"),
            OsStr::new("--disable-setuid-sandbox"),
            OsStr::new("--disable-extensions"),
            OsStr::new("--user-agent=Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.0 Safari/537.36")
        ])
        .build()?;

    let browser = Browser::new(launch_opts)?;
    let tab = browser.new_tab()?;

    // Step 1: perform search
    let mut results = search(query, &tab)?;

    // Step 2: fetch content
    for r in results.iter_mut() {
        match fetch_content(&tab, &r.url) {
            Ok(html) => r.content = Some(html),
            Err(e) => {
                eprintln!("[ERROR] Error fetching {}: {}", r.url, e);
                r.content = Some(format!("Failed to fetch content: {}", e));
            }
        }
    }

    // Step 3: save to Markdown
    save_results_markdown(query, &results, output_folder)?;
    Ok(())
}

