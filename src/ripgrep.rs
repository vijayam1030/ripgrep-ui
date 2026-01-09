use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: PathBuf,
    pub line_number: usize,
    pub column: usize,
    pub line_content: String,
    pub matches: Vec<Match>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct RipgrepBuilder {
    pattern: String,
    path: String,
    ignore_case: bool,
    smart_case: bool,
    hidden: bool,
    file_types: Vec<String>,
    glob: Vec<String>,
    max_depth: Option<usize>,
    max_results: Option<usize>,
    additional_args: Vec<String>,
}

impl RipgrepBuilder {
    pub fn new(pattern: String) -> Self {
        Self {
            pattern,
            path: ".".to_string(),
            ignore_case: false,
            smart_case: true,
            hidden: false,
            file_types: Vec::new(),
            glob: Vec::new(),
            max_depth: None,
            max_results: Some(1000),
            additional_args: Vec::new(),
        }
    }

    pub fn path(mut self, path: String) -> Self {
        self.path = path;
        self
    }

    pub fn ignore_case(mut self, ignore_case: bool) -> Self {
        self.ignore_case = ignore_case;
        self
    }

    pub fn smart_case(mut self, smart_case: bool) -> Self {
        self.smart_case = smart_case;
        self
    }

    pub fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    pub fn file_type(mut self, file_type: String) -> Self {
        self.file_types.push(file_type);
        self
    }

    pub fn glob_pattern(mut self, glob: String) -> Self {
        self.glob.push(glob);
        self
    }

    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    pub fn max_results(mut self, max: usize) -> Self {
        self.max_results = Some(max);
        self
    }

    pub fn apply_preset(mut self, config: &Config, preset_name: &str) -> Result<Self> {
        let preset = config.presets.get(preset_name)
            .ok_or_else(|| anyhow::anyhow!("Preset '{}' not found", preset_name))?;

        if let Some(pattern) = &preset.pattern {
            self.pattern = pattern.clone();
        }
        
        self.ignore_case = preset.ignore_case;
        self.hidden = preset.hidden;
        self.file_types = preset.file_types.clone();
        self.glob = preset.glob.clone();
        self.max_depth = preset.max_depth;
        self.additional_args = preset.additional_args.clone();

        Ok(self)
    }

    pub fn execute(&self) -> Result<Vec<SearchResult>> {
        let mut cmd = Command::new("rg");
        
        // JSON output for parsing
        cmd.arg("--json");
        
        // Case sensitivity
        if self.ignore_case {
            cmd.arg("--ignore-case");
        } else if self.smart_case {
            cmd.arg("--smart-case");
        }

        // Hidden files
        if self.hidden {
            cmd.arg("--hidden");
        }

        // File types
        for ft in &self.file_types {
            cmd.arg("-t").arg(ft);
        }

        // Glob patterns
        for g in &self.glob {
            cmd.arg("-g").arg(g);
        }

        // Max depth
        if let Some(depth) = self.max_depth {
            cmd.arg("--max-depth").arg(depth.to_string());
        }

        // Max count (limit results per file)
        if let Some(max) = self.max_results {
            cmd.arg("--max-count").arg(max.to_string());
        }

        // Additional arguments
        for arg in &self.additional_args {
            cmd.arg(arg);
        }

        // Pattern and path
        cmd.arg(&self.pattern);
        cmd.arg(&self.path);

        // Execute
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let output = cmd.output()
            .context("Failed to execute ripgrep. Make sure 'rg' is installed and in PATH.")?;

        if !output.status.success() && output.status.code() != Some(1) {
            // Exit code 1 means no matches found, which is okay
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Ripgrep error: {}", stderr);
        }

        // Parse JSON output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let results = self.parse_json_output(&stdout)?;

        Ok(results)
    }

    fn parse_json_output(&self, output: &str) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let value: serde_json::Value = serde_json::from_str(line)
                .context("Failed to parse ripgrep JSON output")?;

            if value["type"] == "match" {
                let data = &value["data"];
                
                let path = PathBuf::from(
                    data["path"]["text"]
                        .as_str()
                        .unwrap_or("")
                );

                let line_number = data["line_number"]
                    .as_u64()
                    .unwrap_or(0) as usize;

                let line_content = data["lines"]["text"]
                    .as_str()
                    .unwrap_or("")
                    .trim_end_matches('\n')
                    .to_string();

                let mut matches = Vec::new();
                if let Some(submatches) = data["submatches"].as_array() {
                    for submatch in submatches {
                        let start = submatch["start"].as_u64().unwrap_or(0) as usize;
                        let end = submatch["end"].as_u64().unwrap_or(0) as usize;
                        let text = submatch["match"]["text"].as_str().unwrap_or("").to_string();
                        
                        matches.push(Match { start, end, text });
                    }
                }

                results.push(SearchResult {
                    path,
                    line_number,
                    column: matches.first().map(|m| m.start).unwrap_or(0),
                    line_content,
                    matches,
                });
            }
        }

        Ok(results)
    }
}

pub fn check_ripgrep_installed() -> Result<bool> {
    match Command::new("rg").arg("--version").output() {
        Ok(output) => Ok(output.status.success()),
        Err(_) => Ok(false),
    }
}

pub fn get_ripgrep_version() -> Result<String> {
    let output = Command::new("rg")
        .arg("--version")
        .output()
        .context("Failed to get ripgrep version")?;
    
    let version = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or("unknown")
        .to_string();
    
    Ok(version)
}
