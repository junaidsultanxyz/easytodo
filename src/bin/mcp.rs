use std::io::{self, BufRead, Write};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use easytodo::config::Config;
use easytodo::errors::AppError;
use easytodo::task::model::{Filter, TaskField};
use easytodo::task::store::FsTaskStore;
use easytodo::task::store::TaskStore;

#[derive(Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    #[serde(default)]
    id: Option<Value>,
    #[serde(default)]
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

fn success(id: Option<Value>, result: Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".into(),
        id,
        result: Some(result),
        error: None,
    }
}

fn error(id: Option<Value>, code: i32, message: String) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".into(),
        id,
        result: None,
        error: Some(JsonRpcError {
            code,
            message,
            data: None,
        }),
    }
}

fn tool_def(name: &str, description: &str, input_schema: Value) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema
    })
}

fn string_schema(description: &str) -> Value {
    json!({"type": "string", "description": description})
}

fn optional_string_schema(description: &str) -> Value {
    json!({"type": "string", "description": description})
}

fn list_tools() -> Value {
    json!({
        "tools": [
            tool_def(
                "list_tasks",
                "List all tasks, optionally filtered by status",
                json!({
                    "type": "object",
                    "properties": {
                        "filter": {
                            "type": "string",
                            "description": "Filter by status: 'all', 'todo', or 'done'",
                            "enum": ["all", "todo", "done"]
                        }
                    }
                })
            ),
            tool_def(
                "get_task",
                "Get a single task by its ID",
                json!({
                    "type": "object",
                    "properties": {
                        "id": string_schema("The task ID (6-character hex string)")
                    },
                    "required": ["id"]
                })
            ),
            tool_def(
                "create_task",
                "Create a new task",
                json!({
                    "type": "object",
                    "properties": {
                        "title": string_schema("Task title"),
                        "description": optional_string_schema("Task description"),
                        "due_date": optional_string_schema("Due date in YYYY-MM-DD format")
                    },
                    "required": ["title"]
                })
            ),
            tool_def(
                "update_task",
                "Update one or more fields of a task",
                json!({
                    "type": "object",
                    "properties": {
                        "id": string_schema("The task ID"),
                        "title": optional_string_schema("New title"),
                        "description": optional_string_schema("New description"),
                        "due_date": optional_string_schema("Due date in YYYY-MM-DD format, or empty string to clear"),
                        "status": optional_string_schema("New status: 'open' or 'done'")
                    },
                    "required": ["id"]
                })
            ),
            tool_def(
                "delete_task",
                "Delete a task by its ID",
                json!({
                    "type": "object",
                    "properties": {
                        "id": string_schema("The task ID")
                    },
                    "required": ["id"]
                })
            ),
            tool_def(
                "done_task",
                "Mark a task as done",
                json!({
                    "type": "object",
                    "properties": {
                        "id": string_schema("The task ID")
                    },
                    "required": ["id"]
                })
            ),
            tool_def(
                "undone_task",
                "Mark a task as not done",
                json!({
                    "type": "object",
                    "properties": {
                        "id": string_schema("The task ID")
                    },
                    "required": ["id"]
                })
            ),
            tool_def(
                "help",
                "Show available commands and shortcuts",
                json!({"type": "object", "properties": {}})
            ),
        ]
    })
}

fn handle_tools_call(name: &str, args: &Value, store: &mut FsTaskStore) -> Result<Value, String> {
    match name {
        "list_tasks" => {
            let filter = args
                .get("filter")
                .and_then(|v| v.as_str())
                .and_then(Filter::from_str)
                .unwrap_or(Filter::All);
            let tasks = store.list(filter).map_err(|e: AppError| e.to_string())?;
            let task_list: Vec<Value> = tasks
                .iter()
                .map(|t| {
                    json!({
                        "id": t.id,
                        "title": t.title,
                        "description": t.description,
                        "status": t.status.to_string(),
                        "created_at": t.created_at.to_rfc3339(),
                        "updated_at": t.updated_at.to_rfc3339(),
                        "due_date": t.due_date.map(|d| d.to_string())
                    })
                })
                .collect();
            Ok(json!({
                "content": [{"type": "text", "text": serde_json::to_string_pretty(&task_list).unwrap_or_default()}]
            }))
        }
        "get_task" => {
            let id = args
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'id' argument")?;
            let task = store.get(id).map_err(|e: AppError| e.to_string())?;
            Ok(json!({
                "content": [{"type": "text", "text": serde_json::to_string_pretty(&task).unwrap_or_default()}]
            }))
        }
        "create_task" => {
            let title = args
                .get("title")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'title' argument")?;
            let description = args.get("description").and_then(|v| v.as_str()).unwrap_or("");
            let due_date = args
                .get("due_date")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .and_then(|s| {
                    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
                });
            let task = store
                .create(title, description, due_date)
                .map_err(|e: AppError| e.to_string())?;
            Ok(json!({
                "content": [{"type": "text", "text": format!("Task created: {} (id: {})", task.title, task.id)}]
            }))
        }
        "update_task" => {
            let id = args
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'id' argument")?;
            let mut updated = false;
            let mut last_error = String::new();

            if let Some(title) = args.get("title").and_then(|v| v.as_str()) {
                match store.update(id, TaskField::Title, title) {
                    Ok(_) => updated = true,
                    Err(e) => last_error = format!("{}", e),
                }
            }
            if let Some(desc) = args.get("description").and_then(|v| v.as_str()) {
                match store.update(id, TaskField::Description, desc) {
                    Ok(_) => updated = true,
                    Err(e) => last_error = format!("{}", e),
                }
            }
            if let Some(due) = args.get("due_date").and_then(|v| v.as_str()) {
                match store.update(id, TaskField::DueDate, due) {
                    Ok(_) => updated = true,
                    Err(e) => last_error = format!("{}", e),
                }
            }
            if let Some(status) = args.get("status").and_then(|v| v.as_str()) {
                match store.update(id, TaskField::Status, status) {
                    Ok(_) => updated = true,
                    Err(e) => last_error = format!("{}", e),
                }
            }

            if updated {
                let task = store.get(id).map_err(|e: AppError| e.to_string())?;
                Ok(json!({
                    "content": [{"type": "text", "text": format!("Task updated: {} (id: {})", task.title, task.id)}]
                }))
            } else if !last_error.is_empty() {
                Err(last_error)
            } else {
                Err("No fields to update".into())
            }
        }
        "delete_task" => {
            let id = args
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'id' argument")?;
            store.delete(id).map_err(|e: AppError| e.to_string())?;
            Ok(json!({
                "content": [{"type": "text", "text": format!("Task deleted: {}", id)}]
            }))
        }
        "done_task" => {
            let id = args
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'id' argument")?;
            store.update(id, TaskField::Status, "done").map_err(|e: AppError| e.to_string())?;
            Ok(json!({
                "content": [{"type": "text", "text": format!("Task marked as done: {}", id)}]
            }))
        }
        "undone_task" => {
            let id = args
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'id' argument")?;
            store.update(id, TaskField::Status, "open").map_err(|e: AppError| e.to_string())?;
            Ok(json!({
                "content": [{"type": "text", "text": format!("Task marked as not done: {}", id)}]
            }))
        }
        "help" => {
            let text = "\
EasyTodo Commands (Ctrl+P):
  new \"<title>\" [\"<desc>\"] [due:YYYY-MM-DD]
    Create a task

  edit <id> [title:\"...\"] [desc:\"...\"] [due:...] [status:...]
    Edit task fields

  done|undone <id>
    Mark done/not done

  delete|clone|open <id>
    Delete, duplicate, or edit task

  list [all|todo|done]
    List tasks with filter

  config [key] [value]
    View or set config

  migrate <path>
    Move task storage

  reload | help | quit
    Reload, show help, or exit

  Aliases: rm=delete  cp=clone  ls=list  mv=migrate  q=quit  refresh=reload
  \".\" resolves to the selected task

Shortcuts:
  j/k/up/down     Navigate
  Enter/Space     Toggle done
  l/o             Open detail
  1/2/3           Filter all/todo/done
  Ctrl+N          New task
  Ctrl+E          Edit task
  Ctrl+D          Delete task
  Ctrl+B          Open config
  Ctrl+H          Show help
  Ctrl+P          Command bar
  Ctrl+Q          Quit
  Esc             Close/back";
            Ok(json!({"content": [{"type": "text", "text": text}]}))
        }
        _ => Err(format!("Unknown tool: {}", name)),
    }
}

fn main() {
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load config: {}", e);
            std::process::exit(1);
        }
    };

    let data_dir = match config.resolved_data_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to resolve data directory: {}", e);
            std::process::exit(1);
        }
    };

    let mut store = match FsTaskStore::new(data_dir) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to initialize store: {}", e);
            std::process::exit(1);
        }
    };

    let stdin = io::stdin();
    let reader = stdin.lock();
    let stdout = io::stdout();
    let mut writer = stdout.lock();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = error(None, -32700, format!("Parse error: {}", e));
                let _ = writeln!(writer, "{}", serde_json::to_string(&resp).unwrap());
                let _ = writer.flush();
                continue;
            }
        };

        let response = match request.method.as_str() {
            "initialize" => {
                let result = json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "easytodo-mcp",
                        "version": "0.1.0"
                    }
                });
                success(request.id, result)
            }
            "notifications/initialized" => continue,
            "tools/list" => success(request.id, list_tools()),
            "tools/call" => {
                let params = request.params.unwrap_or(json!({}));
                let name = params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let args = params.get("arguments").cloned().unwrap_or(json!({}));

                match handle_tools_call(&name, &args, &mut store) {
                    Ok(result) => success(request.id, result),
                    Err(msg) => error(request.id, -32603, msg),
                }
            }
            _ => error(
                request.id,
                -32601,
                format!("Method not found: {}", request.method),
            ),
        };

        let output = serde_json::to_string(&response).unwrap();
        let _ = writeln!(writer, "{}", output);
        let _ = writer.flush();
    }
}
