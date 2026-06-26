use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::process::Command;

/// CLI Arguments Definition
#[derive(Parser, Debug)]
#[command(author, version, about = "Summarize git logs for management using Gemini AI", long_about = None)]
struct Cli {
    /// Path to the git repository
    #[arg(long, default_value = ".")]
    path: String,

    /// Git log since period
    #[arg(long, default_value = "2 years ago")]
    period: String,

    /// Only save the raw filtered git log to gitlog.md and exit
    #[arg(long)]
    only_file: bool,

    /// Disable all stdout prints. Forces output to gitlog.md and result.md
    #[arg(long)]
    silent: bool,

    /// Output destination for the AI summary
    #[arg(long, value_enum, default_value_t = OutputMode::Console)]
    output: OutputMode,

    /// Output language for the summary (e.g., EN, RU, UA)
    #[arg(short, long, default_value = "EN")]
    lang: String,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
enum OutputMode {
    Console,
    File,
    Both,
}

// --- Gemini API Request/Response Structs ---

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiRequest {
    system_instruction: SystemInstruction,
    contents: Vec<Content>,
}

#[derive(Serialize)]
struct SystemInstruction {
    parts: Vec<Part>,
}

#[derive(Serialize, Deserialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize, Deserialize)]
struct Part {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Content,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 1. Initialization & Configuration
    let api_key = if !cli.only_file {
        Some(get_or_prompt_api_key(cli.silent)?)
    } else {
        None
    };

    // 2. Git Execution
    if !cli.silent {
        println!("Fetching git logs from '{}' since '{}'...", cli.path, cli.period);
    }
    let git_log = get_git_log(&cli.path, &cli.period)?;

    if git_log.trim().is_empty() {
        if !cli.silent {
            println!("Warning: Git log is empty for the specified period. Exiting.");
        }
        return Ok(());
    }

    // 3. Output Management (Raw Log)
    if cli.only_file || cli.silent {
        fs::write("gitlog.md", &git_log).context("Failed to write gitlog.md")?;
        if !cli.silent {
            println!("Raw git log saved to gitlog.md");
        }
    }

    if cli.only_file {
        return Ok(());
    }

    // Resolve Language Name (Case-insensitive mapping)
    // FIX: We use `_` to catch the fallback and return a reference to the original `cli.lang`
    // which lives for the entire duration of the `main` function.
    let target_language = match cli.lang.to_uppercase().as_str() {
        "RU" => "Russian",
        "UA" => "Ukrainian",
        "EN" => "English",
        _ => cli.lang.as_str(),
    };

    // 4. Gemini API Integration
    if !cli.silent {
        println!("Sending logs to Gemini API for summarization (Language: {})...", target_language);
    }

    let summary = summarize_log(api_key.as_deref().unwrap(), &git_log, target_language).await?;

    // 5. Output Management (AI Result)
    let final_output_mode = if cli.silent {
        OutputMode::File
    } else {
        cli.output
    };

    match final_output_mode {
        OutputMode::Console => {
            println!("\n--- Management Summary ---\n{}\n--------------------------", summary);
        }
        OutputMode::File => {
            fs::write("result.md", &summary).context("Failed to write result.md")?;
            if !cli.silent {
                println!("Summary saved to result.md");
            }
        }
        OutputMode::Both => {
            fs::write("result.md", &summary).context("Failed to write result.md")?;
            println!("\n--- Management Summary ---\n{}\n--------------------------", summary);
            println!("Summary also saved to result.md");
        }
    }

    Ok(())
}

/// Checks for GEMINI_API_KEY in env or .env file. Prompts the user if missing.
fn get_or_prompt_api_key(silent: bool) -> Result<String> {
    let _ = dotenvy::dotenv();

    if let Ok(key) = env::var("GEMINI_API_KEY") {
        if !key.trim().is_empty() {
            return Ok(key);
        }
    }

    if silent {
        anyhow::bail!("GEMINI_API_KEY not found. Cannot prompt the user in --silent mode.");
    }

    print!("GEMINI_API_KEY not found. Please enter your Gemini API Key: ");
    io::stdout().flush()?;

    let mut key = String::new();
    io::stdin().read_line(&mut key).context("Failed to read from stdin")?;
    let key = key.trim().to_string();

    if key.is_empty() {
        anyhow::bail!("API key cannot be empty.");
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(".env")
        .context("Failed to open or create .env file")?;

    writeln!(file, "GEMINI_API_KEY={}", key).context("Failed to write to .env file")?;

    unsafe {
        env::set_var("GEMINI_API_KEY", &key);
    }

    Ok(key)
}

/// Executes the git log command and returns the stdout
fn get_git_log(path: &str, period: &str) -> Result<String> {
    let status = Command::new("git")
        .args(["-C", path, "rev-parse", "--is-inside-work-tree"])
        .output()
        .context("Failed to execute git command. Is git installed?")?;

    if !status.status.success() {
        anyhow::bail!("The specified path '{}' is not a valid git repository.", path);
    }

    let output = Command::new("git")
        .args([
            "-C", path,
            "log",
            &format!("--since={}", period),
            "--no-merges",
            "--name-status",
            "--pretty=format:COMMIT: %s (%ad)",
            "--date=short"
        ])
        .output()
        .context("Failed to execute git log command")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git log command failed: {}", err);
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Calls the Gemini 2.5 Flash API to summarize the git log in the requested language
async fn summarize_log(api_key: &str, log_data: &str, language: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let system_prompt = format!(
        "You are an expert technical analyst. Read the provided git log. \
        Ignore routine or minor fixes. Group the changes by high-level business features \
        or modules (e.g., 'Billing System Update', 'Refactoring'). \
        Highlight significant changes on the surface. Exclude any sales operations if visible. \
        The final summary must be non-technical, well-structured, and suited for management reporting. \
        You must generate the entire summary and report strictly in the {} language.",
        language
    );

    let req_body = GeminiRequest {
        system_instruction: SystemInstruction {
            parts: vec![Part { text: system_prompt }],
        },
        contents: vec![Content {
            parts: vec![Part { text: log_data.to_string() }],
        }],
    };

    let res = client.post(&url)
        .header("Content-Type", "application/json")
        .json(&req_body)
        .send()
        .await
        .context("Failed to send request to Gemini API")?;

    if !res.status().is_success() {
        let err_text = res.text().await?;
        anyhow::bail!("Gemini API returned an error: {}", err_text);
    }

    let response_data: GeminiResponse = res.json().await.context("Failed to parse Gemini API response")?;

    let text = response_data.candidates
        .and_then(|mut c| c.pop())
        .and_then(|c| c.content.parts.into_iter().next())
        .map(|p| p.text)
        .unwrap_or_else(|| "No summary generated.".to_string());

    Ok(text)
}