use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CliOptions {
    pub project: Option<PathBuf>,
    pub read_only: bool,
    pub theme: Option<String>,
    pub config: Option<PathBuf>,
}

impl CliOptions {
    pub fn parse() -> Result<Self, String> {
        let mut args = env::args().skip(1);
        let mut project = None;
        let mut read_only = false;
        let mut theme = None;
        let mut config = None;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    print_help();
                    std::process::exit(0);
                }
                "--read-only" => read_only = true,
                "--project" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--project requires a path".to_string())?;
                    project = Some(PathBuf::from(value));
                }
                "--theme" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--theme requires a name".to_string())?;
                    theme = Some(value);
                }
                "--config" => {
                    let value = args
                        .next()
                        .ok_or_else(|| "--config requires a path".to_string())?;
                    config = Some(PathBuf::from(value));
                }
                other if other.starts_with('-') => {
                    return Err(format!("unknown flag: {other}"));
                }
                other => {
                    if project.is_some() {
                        return Err("only one project path can be provided".to_string());
                    }
                    project = Some(PathBuf::from(other));
                }
            }
        }

        Ok(Self {
            project,
            read_only,
            theme,
            config,
        })
    }
}

fn print_help() {
    println!(
        "\
zendroid 0.1.0

Usage:
  zendroid
  zendroid <project-path>
  zendroid --project <project-path> [--read-only] [--theme <name>] [--config <path>]

Flags:
  --project <path>  Open a specific Android project
  --read-only       Disable edits and task execution
  --theme <name>    Override the configured theme
  --config <path>   Load configuration from a custom path
  -h, --help        Show this help
"
    );
}
