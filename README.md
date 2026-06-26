# Git AI Analyzer 🚀

A lightweight, blazing-fast CLI tool written in Rust that transforms chaotic Git commit histories and diffs into clean, high-level business summaries for management and clients. Powered by Google Gemini AI.

No more manual changelogs or trying to decipher cryptic commits like `fix`, `add migrate`, or `done`.

---

## ✨ Features

* **Smart Context Gathering**: Extracts not only commit messages but also tracks modified files (`--name-status`), allowing the AI to understand the real essence of changes even if commit messages are messy.
* **Massive Context Window**: Utilizes Google Gemini (Gemini 2.5 Flash/Pro), effortlessly handling massive logs for periods up to 2+ years.
* **Multi-language Output**: Generate your reports in **RU, UA, or EN** on the fly using a single flag.
* **Flexible Output Management**: Print directly to the console, save to Markdown files, or both.
* **Zero Global Configuration Hassle**: Automatically prompts for the API key on the first run and securely stores it locally.

---

## 🛠️ Installation & Setup

### Prerequisites
Make sure you have [Rust and Cargo](https://rustup.rs/) installed.

### 1. Build from Source
Clone the repository and build the release binary:
    git clone [https://github.com/tabbakka-developer/git-ai-analyzer.git](https://github.com/tabbakka-developer/git-ai-analyzer.git)
    cd git-ai-analyzer
    cargo build --release

### 2. Make it Global
Move the compiled binary to your local binaries directory to use it anywhere across your system:

* **macOS / Linux**:
    mv target/release/git-analyzer /usr/local/bin/git-analyzer

* **Windows**:
  Move `target/release/git-analyzer.exe` to a folder (e.g., `C:\tools\`) and add that folder to your system's `PATH` environment variable.

---

## 🔑 Getting an API Key

1. Go to [Google AI Studio](https://aistudio.google.com/).
2. Click **"Get API key"** and create a new key.
3. The first time you run `git-analyzer` in any repository, it will prompt you to paste this key and automatically save it to a local `.env` file.

> ⚠️ **Note:** Remember to add `.env` to your global or project `.gitignore` to prevent accidentally leaking your API key!

---

## 🚀 Usage & Examples

Go to **any** Git repository on your machine and run the tool.

### Basic Usage (Generates a summary for the last 2 years in English)
    git-analyzer

### Generate a Report for the Last 2 Weeks in Russian
    git-analyzer --period "2 weeks ago" --lang RU --output both

### Silent Mode (Saves logs to `gitlog.md` and AI results to `result.md` without console output)
    git-analyzer --period "6 months ago" --silent

### Raw Extract Only (Saves the filtered git log to a file without calling AI)
    git-analyzer --only-file

---

## ⚙️ CLI Arguments Reference

| Flag / Option | Default | Description |
| --- | --- | --- |
| `--path <PATH>` | `.` | Path to the target Git repository. |
| `--period <PERIOD>`| `2 years ago` | Time period for Git log (e.g., `"1 month ago"`, `"2 weeks ago"`). |
| `--lang <LANG>` | `EN` | Output language for the AI summary (`EN`, `RU`, `UA`). |
| `--output <TYPE>` | `console` | Where to output the result (`console`, `file`, `both`). |
| `--only-file` | *None* | Flags the tool to only extract raw logs to `gitlog.md` and exit. |
| `--silent` | *None* | Disables all stdout console prints. Forces writing directly to files. |

---

## 🛡️ License

This project is licensed under the MIT License - see the LICENSE file for details.

---
Driven by passion, built with **Rust** 🦀
