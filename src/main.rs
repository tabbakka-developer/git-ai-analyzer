use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use directories::ProjectDirs;
use inquire::{Select, Text};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

// ==========================================
// CLI & Enums
// ==========================================

#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "Summarize git logs for management using AI", long_about = None)]
struct Cli {
    /// Path to the git repository
    #[arg(long, default_value = ".")]
    path: String,

    /// Git log since period
    #[arg(long, default_value = "2 weeks ago")]
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

    /// Analysis depth mode
    #[arg(short, long, value_enum, default_value_t = AnalysisMode::Medium)]
    mode: AnalysisMode,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
enum OutputMode {
    Console,
    File,
    Both,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
enum AnalysisMode {
    Light,
    Medium,
    Deep,
}

// ==========================================
// Configuration Structs (TOML)
// ==========================================

#[derive(Serialize, Deserialize, Default)]
struct AppConfig {
    current_provider: String,
    providers: Providers,
}

#[derive(Serialize, Deserialize, Default)]
struct Providers {
    gemini: Option<ProviderConfig>,
    openai: Option<ProviderConfig>,
}

#[derive(Serialize, Deserialize)]
struct ProviderConfig {
    api_key: String,
    model: String,
}

// ==========================================
// Main Execution
// ==========================================

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Determine if we should run the TUI (no args passed)
    let args: Vec<String> = env::args().collect();
    let run_tui = args.len() == 1;

    // 2. Load or Setup Configuration
    let config = load_or_setup_config(run_tui)?;

    // 3. Parse CLI or Run TUI to get parameters
    let cli = if run_tui {
        run_interactive_tui()?
    } else {
        Cli::parse()
    };

    // 4. Git Execution based on Mode
    if !cli.silent {
        println!(
            "Fetching git logs from '{}' since '{}' (Mode: {:?})...",
            cli.path, cli.period, cli.mode
        );
    }
    let git_log = get_git_log(&cli.path, &cli.period, &cli.mode)?;

    if git_log.trim().is_empty() {
        if !cli.silent {
            println!("Warning: Git log is empty for the specified period. Exiting.");
        }
        return Ok(());
    }

    // 5. Output Management (Raw Log)
    if cli.only_file || cli.silent {
        fs::write("gitlog.md", &git_log).context("Failed to write gitlog.md")?;
        if !cli.silent {
            println!("Raw git log saved to gitlog.md");
        }
    }

    if cli.only_file {
        return Ok(());
    }

    // Resolve Language Name
    let target_language = match cli.lang.to_uppercase().as_str() {
        "RU" => "Russian",
        "UA" => "Ukrainian",
        "EN" => "English",
        _ => cli.lang.as_str(),
    };

    // 6. AI Integration
    if !cli.silent {
        println!(
            "Sending logs to {} for summarization (Language: {})...",
            config.current_provider, target_language
        );
    }

    let summary = summarize_log(&config, &git_log, target_language, &cli.mode).await?;

    // 7. Output Management (AI Result)
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

// ==========================================
// Configuration & Setup Wizard
// ==========================================

fn get_config_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("", "", "git-analyzer")
        .context("Could not determine home directory")?;
    let config_dir = proj_dirs.config_dir();
    if !config_dir.exists() {
        fs::create_dir_all(config_dir)?;
    }
    Ok(config_dir.join("config.toml"))
}

fn load_or_setup_config(allow_interactive: bool) -> Result<AppConfig> {
    let config_path = get_config_path()?;

    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        if let Ok(config) = toml::from_str::<AppConfig>(&content) {
            // Basic validation to ensure the current provider has a key
            let has_key = match config.current_provider.as_str() {
                "gemini" => config.providers.gemini.is_some(),
                "openai" => config.providers.openai.is_some(),
                _ => false,
            };
            if has_key {
                return Ok(config);
            }
        }
    }

    if !allow_interactive {
        anyhow::bail!("Configuration missing or invalid. Run the tool without arguments to trigger the setup wizard.");
    }

    // Setup Wizard
    println!("Welcome! No valid configuration or API key found.");
    let provider_choice = Select::new(
        "Which provider do you want to configure first?",
        vec!["gemini", "openai"],
    )
        .prompt()?;

    let api_key = Text::new(&format!("Enter your {} API Key:", provider_choice)).prompt()?;

    let mut config = AppConfig {
        current_provider: provider_choice.to_string(),
        providers: Providers::default(),
    };

    if provider_choice == "gemini" {
        config.providers.gemini = Some(ProviderConfig {
            api_key,
            model: "gemini-2.5-flash".to_string(),
        });
    } else {
        config.providers.openai = Some(ProviderConfig {
            api_key,
            model: "gpt-4o-mini".to_string(),
        });
    }

    let toml_string = toml::to_string(&config)?;
    fs::write(&config_path, toml_string)?;
    println!("Configuration saved to {:?}\n", config_path);

    Ok(config)
}

// ==========================================
// Interactive TUI
// ==========================================

fn run_interactive_tui() -> Result<Cli> {
    let period_opts = vec!["1 week ago", "2 weeks ago", "1 month ago", "Custom"];
    let mut period = Select::new("Select time period:", period_opts).prompt()?.to_string();

    if period == "Custom" {
        period = Text::new("Enter custom period (e.g., '3 days ago'):").prompt()?;
    }

    let lang = Select::new("Select output language:", vec!["EN", "RU", "UA"]).prompt()?.to_string();

    let mode_str = Select::new("Select analysis depth:", vec!["light", "medium", "deep"]).prompt()?;
    let mode = match mode_str {
        "light" => AnalysisMode::Light,
        "deep" => AnalysisMode::Deep,
        _ => AnalysisMode::Medium,
    };

    let output_str = Select::new("Select output target:", vec!["console", "file", "both"]).prompt()?;
    let output = match output_str {
        "file" => OutputMode::File,
        "both" => OutputMode::Both,
        _ => OutputMode::Console,
    };

    Ok(Cli {
        path: ".".to_string(),
        period,
        only_file: false,
        silent: false,
        output,
        lang,
        mode,
    })
}

// ==========================================
// Git Execution
// ==========================================

fn get_git_log(path: &str, period: &str, mode: &AnalysisMode) -> Result<String> {
    let status = Command::new("git")
        .args(["-C", path, "rev-parse", "--is-inside-work-tree"])
        .output()
        .context("Failed to execute git command. Is git installed?")?;

    if !status.status.success() {
        anyhow::bail!("The specified path '{}' is not a valid git repository.", path);
    }

    let mut args = vec![
        "-C".to_string(),
        path.to_string(),
        "log".to_string(),
        format!("--since={}", period),
        "--no-merges".to_string(),
    ];

    match mode {
        AnalysisMode::Light => {
            args.push("--oneline".to_string());
        }
        AnalysisMode::Medium => {
            args.push("--name-status".to_string());
            args.push("--pretty=format:COMMIT: %s (%ad)".to_string());
            args.push("--date=short".to_string());
        }
        AnalysisMode::Deep => {
            args.push("--name-status".to_string());
            args.push("--pretty=format:COMMIT: %s (%ad)".to_string());
            args.push("--date=short".to_string());
            args.push("-p".to_string());
            args.push("-U0".to_string()); // Zero context lines to save tokens
            args.push("--".to_string());
            args.push(".".to_string());
            // Exclude heavy/useless files from the diff
            args.push(":(exclude)*lock*".to_string());
            args.push(":(exclude)*.lock".to_string());
            args.push(":(exclude)vendor/".to_string());
            args.push(":(exclude)node_modules/".to_string());
            args.push(":(exclude)dist/".to_string());
        }
    }

    let output = Command::new("git")
        .args(&args)
        .output()
        .context("Failed to execute git log command")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git log command failed: {}", err);
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

// ==========================================
// AI Provider Integration
// ==========================================

async fn summarize_log(
    config: &AppConfig,
    log_data: &str,
    language: &str,
    mode: &AnalysisMode,
) -> Result<String> {
    let depth_instruction = match mode {
        AnalysisMode::Light => "You are provided with a high-level list of commit messages.",
        AnalysisMode::Medium => "You are provided with commit messages and the files modified.",
        AnalysisMode::Deep => "You are provided with commit messages, modified files, and structural code diffs. Use the diffs to understand core logic changes (e.g., new API routes, database migrations).",
    };

    let system_prompt = format!(
        "You are an expert technical analyst. Read the provided git log. \
        {} \
        Ignore routine or minor fixes. Group the changes by high-level business features \
        or modules (e.g., 'Billing System Update', 'Refactoring'). \
        Highlight significant changes on the surface. Exclude any sales operations if visible. \
        The final summary must be non-technical, well-structured, and suited for management reporting. \
        You must generate the entire summary and report strictly in the {} language.",
        depth_instruction, language
    );

    match config.current_provider.as_str() {
        "gemini" => {
            let provider_cfg = config.providers.gemini.as_ref().context("Gemini config missing")?;
            call_gemini(&provider_cfg.api_key, &provider_cfg.model, &system_prompt, log_data).await
        }
        "openai" => {
            let provider_cfg = config.providers.openai.as_ref().context("OpenAI config missing")?;
            call_openai(&provider_cfg.api_key, &provider_cfg.model, &system_prompt, log_data).await
        }
        other => anyhow::bail!("Unsupported provider: {}", other),
    }
}

// --- Gemini Implementation ---

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiRequest {
    system_instruction: GeminiSystemInstruction,
    contents: Vec<GeminiContent>,
}
#[derive(Serialize)]
struct GeminiSystemInstruction { parts: Vec<GeminiPart> }
#[derive(Serialize, Deserialize)]
struct GeminiContent { parts: Vec<GeminiPart> }
#[derive(Serialize, Deserialize)]
struct GeminiPart { text: String }
#[derive(Deserialize)]
struct GeminiResponse { candidates: Option<Vec<GeminiCandidate>> }
#[derive(Deserialize)]
struct GeminiCandidate { content: GeminiContent }

async fn call_gemini(api_key: &str, model: &str, system_prompt: &str, log_data: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        model, api_key
    );

    let req_body = GeminiRequest {
        system_instruction: GeminiSystemInstruction {
            parts: vec![GeminiPart { text: system_prompt.to_string() }],
        },
        contents: vec![GeminiContent {
            parts: vec![GeminiPart { text: log_data.to_string() }],
        }],
    };

    let res = client.post(&url).json(&req_body).send().await?;
    if !res.status().is_success() {
        anyhow::bail!("Gemini API error: {}", res.text().await?);
    }

    let response_data: GeminiResponse = res.json().await?;
    let text = response_data.candidates
        .and_then(|mut c| c.pop())
        .and_then(|c| c.content.parts.into_iter().next())
        .map(|p| p.text)
        .unwrap_or_else(|| "No summary generated.".to_string());

    Ok(text)
}

// --- OpenAI Implementation ---

#[derive(Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
}
#[derive(Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}
#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Option<Vec<OpenAiChoice>>,
}
#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

async fn call_openai(api_key: &str, model: &str, system_prompt: &str, log_data: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let url = "https://api.openai.com/v1/chat/completions";

    let req_body = OpenAiRequest {
        model: model.to_string(),
        messages: vec![
            OpenAiMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            OpenAiMessage {
                role: "user".to_string(),
                content: log_data.to_string(),
            },
        ],
    };

    let res = client.post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&req_body)
        .send()
        .await?;

    if !res.status().is_success() {
        anyhow::bail!("OpenAI API error: {}", res.text().await?);
    }

    let response_data: OpenAiResponse = res.json().await?;
    let text = response_data.choices
        .and_then(|mut c| c.pop())
        .map(|c| c.message.content)
        .unwrap_or_else(|| "No summary generated.".to_string());

    Ok(text)
}