use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    #[serde(skip)]
    pub description: String,
    pub status: Status,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub due_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Status {
    #[serde(rename = "open")]
    Open,
    #[serde(rename = "done")]
    Done,
}

impl Status {
    pub fn from_str(s: &str) -> Option<Status> {
        match s.to_lowercase().as_str() {
            "open" => Some(Status::Open),
            "done" => Some(Status::Done),
            _ => None,
        }
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Open => write!(f, "open"),
            Status::Done => write!(f, "done"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Filter {
    All,
    Todo,
    Done,
}

impl Filter {
    pub fn from_str(s: &str) -> Option<Filter> {
        match s.to_lowercase().as_str() {
            "all" => Some(Filter::All),
            "todo" | "open" => Some(Filter::Todo),
            "done" => Some(Filter::Done),
            _ => None,
        }
    }
}

impl std::fmt::Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Filter::All => write!(f, "all"),
            Filter::Todo => write!(f, "todo"),
            Filter::Done => write!(f, "done"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskField {
    Title,
    Description,
    DueDate,
    Status,
}

impl TaskField {
    pub fn from_str(s: &str) -> Option<TaskField> {
        match s.to_lowercase().as_str() {
            "title" => Some(TaskField::Title),
            "description" | "desc" => Some(TaskField::Description),
            "due" | "due_date" | "duedate" => Some(TaskField::DueDate),
            "status" => Some(TaskField::Status),
            _ => None,
        }
    }
}
