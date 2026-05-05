use crate::commands::Command;
use crate::errors::{AppError, Result};

pub fn tokenize(input: &str) -> Result<Vec<String>> {
    let input = input.trim();
    if input.is_empty() {
        return Ok(Vec::new());
    }

    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }

        if chars[i] == '"' || chars[i] == '\'' {
            let quote = chars[i];
            i += 1;
            let mut s = String::new();
            while i < chars.len() && chars[i] != quote {
                s.push(chars[i]);
                i += 1;
            }
            if i >= chars.len() {
                return Err(AppError::ParseError(format!(
                    "Unterminated {} quote",
                    quote
                )));
            }
            i += 1;
            tokens.push(s);
        } else {
            let mut s = String::new();
            while i < chars.len() && !chars[i].is_whitespace() {
                if chars[i] == '"' || chars[i] == '\'' {
                    let quote = chars[i];
                    i += 1;
                    while i < chars.len() && chars[i] != quote {
                        s.push(chars[i]);
                        i += 1;
                    }
                    if i >= chars.len() {
                        return Err(AppError::ParseError(format!(
                            "Unterminated {} quote",
                            quote
                        )));
                    }
                    i += 1;
                } else {
                    s.push(chars[i]);
                    i += 1;
                }
            }
            tokens.push(s);
        }
    }

    Ok(tokens)
}

pub fn parse(input: &str) -> Result<Command> {
    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        return Err(AppError::CommandError("No command entered".into()));
    }

    let cmd_name = tokens[0].to_lowercase();

    match cmd_name.as_str() {
        "new" => parse_new(&tokens),
        "open" => parse_one_arg(&tokens, "open"),
        "edit" => parse_edit(&tokens),
        "delete" | "rm" => parse_one_arg(&tokens, "delete"),
        "done" => parse_one_arg(&tokens, "done"),
        "undone" => parse_one_arg(&tokens, "undone"),
        "clone" | "cp" => parse_one_arg(&tokens, "clone"),
        "list" | "ls" => parse_list(&tokens),
        "config" => parse_config(&tokens),
        "migrate" | "mv" => parse_one_arg(&tokens, "migrate"),
        "reload" | "refresh" => Ok(Command::Reload),
        "help" => Ok(Command::Help),
        "quit" | "q" | "exit" => Ok(Command::Quit),
        _ => Err(AppError::CommandError(format!(
            "Unknown command: '{}'",
            cmd_name
        ))),
    }
}

fn parse_new(tokens: &[String]) -> Result<Command> {
    if tokens.len() < 2 {
        return Err(AppError::CommandError(
            "Usage: new \"<title>\" [\"<description>\"] [\"<due-date>\"]".into(),
        ));
    }
    let title = tokens[1].clone();
    let description = tokens.get(2).cloned().unwrap_or_default();
    let due = tokens.get(3).cloned();
    Ok(Command::New {
        title,
        description,
        due,
    })
}

fn parse_one_arg(tokens: &[String], cmd: &str) -> Result<Command> {
    if tokens.len() < 2 {
        return Err(AppError::CommandError(format!(
            "Usage: {} \"<id>\"",
            cmd
        )));
    }
    let id = tokens[1].clone();
    match cmd {
        "open" => Ok(Command::Open(id)),
        "delete" => Ok(Command::Delete(id)),
        "done" => Ok(Command::Done(id)),
        "undone" => Ok(Command::Undone(id)),
        "clone" => Ok(Command::Clone(id)),
        "migrate" => Ok(Command::Migrate(id)),
        _ => unreachable!(),
    }
}

fn parse_edit(tokens: &[String]) -> Result<Command> {
    if tokens.len() < 2 {
        return Err(AppError::CommandError(
            "Usage: edit <id> title:\"new title\" description:\"desc\"".into(),
        ));
    }
    let id = tokens[1].clone();
    let mut fields = Vec::new();
    for token in &tokens[2..] {
        if let Some(pos) = token.find(':') {
            let key = token[..pos].to_string();
            let value = token[pos + 1..].to_string();
            fields.push((key, value));
        }
    }
    Ok(Command::Edit { id, fields })
}

fn parse_list(tokens: &[String]) -> Result<Command> {
    let filter = tokens.get(1).cloned();
    Ok(Command::List(filter))
}

fn parse_config(tokens: &[String]) -> Result<Command> {
    let key = tokens.get(1).cloned();
    let value = tokens.get(2).cloned();
    Ok(Command::Config { key, value })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let tokens = tokenize(r#"new "Buy milk" "groceries" "2026-05-10""#).unwrap();
        assert_eq!(tokens, vec!["new", "Buy milk", "groceries", "2026-05-10"]);
    }

    #[test]
    fn test_tokenize_single_quotes() {
        let tokens = tokenize(r#"open 'abc123'"#).unwrap();
        assert_eq!(tokens, vec!["open", "abc123"]);
    }

    #[test]
    fn test_tokenize_empty_string() {
        let tokens = tokenize(r#"new "title" "" "2026-05-10""#).unwrap();
        assert_eq!(tokens, vec!["new", "title", "", "2026-05-10"]);
    }

    #[test]
    fn test_tokenize_no_quotes() {
        let tokens = tokenize("list").unwrap();
        assert_eq!(tokens, vec!["list"]);
    }

    #[test]
    fn test_tokenize_unterminated() {
        assert!(tokenize(r#"new "title"#).is_err());
    }

    #[test]
    fn test_parse_new_title_only() {
        let cmd = parse(r#"new "My Task""#).unwrap();
        assert_eq!(
            cmd,
            Command::New {
                title: "My Task".into(),
                description: String::new(),
                due: None
            }
        );
    }

    #[test]
    fn test_parse_new_full() {
        let cmd = parse(r#"new "Task" "Desc" "2026-05-10""#).unwrap();
        assert_eq!(
            cmd,
            Command::New {
                title: "Task".into(),
                description: "Desc".into(),
                due: Some("2026-05-10".into())
            }
        );
    }

    #[test]
    fn test_parse_new_no_args() {
        assert!(parse("new").is_err());
    }

    #[test]
    fn test_parse_done() {
        let cmd = parse(r#"done "abc123""#).unwrap();
        assert_eq!(cmd, Command::Done("abc123".into()));
    }

    #[test]
    fn test_parse_list() {
        let cmd = parse("list").unwrap();
        assert_eq!(cmd, Command::List(None));

        let cmd = parse(r#"list "done""#).unwrap();
        assert_eq!(cmd, Command::List(Some("done".into())));
    }

    #[test]
    fn test_parse_edit_single_field() {
        let cmd = parse("edit abc title:\"New Title\"").unwrap();
        assert_eq!(
            cmd,
            Command::Edit {
                id: "abc".into(),
                fields: vec![("title".into(), "New Title".into())]
            }
        );
    }

    #[test]
    fn test_parse_edit_multi_field() {
        let cmd = parse("edit abc title:\"New Title\" due_date:2026-05-10").unwrap();
        assert_eq!(
            cmd,
            Command::Edit {
                id: "abc".into(),
                fields: vec![
                    ("title".into(), "New Title".into()),
                    ("due_date".into(), "2026-05-10".into())
                ]
            }
        );
    }

    #[test]
    fn test_parse_edit_no_fields() {
        let cmd = parse("edit abc123").unwrap();
        assert_eq!(
            cmd,
            Command::Edit {
                id: "abc123".into(),
                fields: vec![]
            }
        );
    }

    #[test]
    fn test_tokenize_inline_quote() {
        let tokens = tokenize("edit abc title:\"new title\" desc:\"hello world\"").unwrap();
        assert_eq!(tokens, vec!["edit", "abc", "title:new title", "desc:hello world"]);
    }

    #[test]
    fn test_tokenize_single_quote_inline() {
        let tokens = tokenize("edit abc title:'my title'").unwrap();
        assert_eq!(tokens, vec!["edit", "abc", "title:my title"]);
    }

    #[test]
    fn test_parse_reload() {
        let cmd = parse("reload").unwrap();
        assert_eq!(cmd, Command::Reload);
        let cmd = parse("refresh").unwrap();
        assert_eq!(cmd, Command::Reload);
    }

    #[test]
    fn test_edit_done_shortcut_still_works() {
        let cmd = parse(r#"done "abc123""#).unwrap();
        assert_eq!(cmd, Command::Done("abc123".into()));
    }

    #[test]
    fn test_parse_shortcuts() {
        assert_eq!(parse("q").unwrap(), Command::Quit);
        assert_eq!(parse("ls").unwrap(), Command::List(None));
        assert_eq!(parse(r#"rm "abc""#).unwrap(), Command::Delete("abc".into()));
    }

    #[test]
    fn test_parse_unknown() {
        assert!(parse("bogus").is_err());
    }

    #[test]
    fn test_parse_empty() {
        assert!(parse("").is_err());
    }

    #[test]
    fn test_parse_config_get() {
        let cmd = parse(r#"config "editor""#).unwrap();
        assert_eq!(
            cmd,
            Command::Config {
                key: Some("editor".into()),
                value: None
            }
        );
    }

    #[test]
    fn test_parse_config_set() {
        let cmd = parse(r#"config "editor" "nvim""#).unwrap();
        assert_eq!(
            cmd,
            Command::Config {
                key: Some("editor".into()),
                value: Some("nvim".into())
            }
        );
    }
}
