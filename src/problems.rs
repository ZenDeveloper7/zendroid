#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProblemSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub severity: ProblemSeverity,
    pub source: String,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<usize>,
}

#[derive(Debug, Default)]
pub struct ProblemsState {
    pub problems: Vec<Problem>,
    pub selected: usize,
}

impl ProblemsState {
    pub fn clear(&mut self) {
        self.problems.clear();
        self.selected = 0;
    }

    pub fn push_from_output(&mut self, line: &str) {
        if let Some(problem) = parse_problem_line(line) {
            self.problems.push(problem);
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.problems.len() {
            self.selected += 1;
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }
}

pub fn parse_problem_line(line: &str) -> Option<Problem> {
    let lower = line.to_lowercase();
    let severity = if lower.contains("error") || line.starts_with("e:") {
        ProblemSeverity::Error
    } else if lower.contains("warning") || line.starts_with("w:") {
        ProblemSeverity::Warning
    } else if lower.contains("info") || line.starts_with("i:") {
        ProblemSeverity::Info
    } else {
        return None;
    };

    let (file, line_number) = parse_file_location(line);
    Some(Problem {
        severity,
        source: "process".to_string(),
        message: line.trim().to_string(),
        file,
        line: line_number,
    })
}

fn parse_file_location(line: &str) -> (Option<String>, Option<usize>) {
    for token in line.split_whitespace() {
        let candidate = token.trim_matches(|ch: char| ch == '(' || ch == ')' || ch == ',');
        if let Some((file, line_number)) = parse_colon_location(candidate) {
            return (Some(file), Some(line_number));
        }
    }
    (None, None)
}

fn parse_colon_location(value: &str) -> Option<(String, usize)> {
    let mut parts = value.rsplitn(3, ':').collect::<Vec<_>>();
    if parts.len() < 2 {
        return None;
    }
    let line_number = parts[1].parse::<usize>().ok()?;
    parts.reverse();
    let file = if parts.len() == 3 {
        parts[0].to_string()
    } else {
        value.rsplit_once(':')?.0.to_string()
    };
    if file.contains('/')
        || file.ends_with(".kt")
        || file.ends_with(".java")
        || file.ends_with(".rs")
    {
        Some((file, line_number))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_error_problem_with_file_location() {
        let problem = parse_problem_line("e: /tmp/App.kt:42:13 Unresolved reference").unwrap();
        assert_eq!(problem.severity, ProblemSeverity::Error);
        assert_eq!(problem.file.as_deref(), Some("/tmp/App.kt"));
        assert_eq!(problem.line, Some(42));
    }

    #[test]
    fn ignores_plain_output() {
        assert!(parse_problem_line("BUILD SUCCESSFUL").is_none());
    }

    #[test]
    fn parses_info_problem() {
        let problem = parse_problem_line("i: generated source was skipped").unwrap();
        assert_eq!(problem.severity, ProblemSeverity::Info);
    }
}
