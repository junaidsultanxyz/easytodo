use std::fs;
use std::path::{Path, PathBuf};

use chrono::{NaiveDate, Utc};
use rand::Rng;

use crate::errors::{AppError, Result};
use crate::task::model::{Filter, Status, Task, TaskField};

pub trait TaskStore: Send + Sync {
    fn list(&self, filter: Filter) -> Result<Vec<Task>>;
    fn get(&self, id: &str) -> Result<Task>;
    fn create(
        &mut self,
        title: &str,
        description: &str,
        due_date: Option<NaiveDate>,
    ) -> Result<Task>;
    fn update(&mut self, id: &str, field: TaskField, value: &str) -> Result<Task>;
    fn delete(&mut self, id: &str) -> Result<()>;
    fn clone_task(&mut self, id: &str) -> Result<Task>;
    fn migrate(&mut self, new_path: &Path) -> Result<()>;
}

pub struct FsTaskStore {
    data_dir: PathBuf,
}

impl FsTaskStore {
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&data_dir)?;
        Ok(FsTaskStore { data_dir })
    }

    fn task_path(&self, id: &str) -> PathBuf {
        self.data_dir.join(format!("{}.md", id))
    }

    fn read_task(&self, path: &Path) -> Result<Task> {
        let content = fs::read_to_string(path)?;
        let mut parts = content.splitn(3, "---");
        parts.next();
        let yaml_str = parts.next().ok_or_else(|| {
            AppError::ParseError("Missing YAML frontmatter".into())
        })?;
        let mut task: Task = serde_yaml::from_str(yaml_str)?;
        task.description = parts.next().unwrap_or("").trim().to_string();
        Ok(task)
    }

    fn write_task(&self, task: &Task) -> Result<()> {
        let yaml_str = serde_yaml::to_string(task)?;
        let content = format!("---\n{}---\n{}", yaml_str, task.description);
        fs::write(self.task_path(&task.id), content)?;
        Ok(())
    }

    fn all_tasks(&self) -> Result<Vec<Task>> {
        let mut tasks = Vec::new();
        let entries = fs::read_dir(&self.data_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                match self.read_task(&path) {
                    Ok(task) => tasks.push(task),
                    Err(_) => continue,
                }
            }
        }
        tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(tasks)
    }

    fn generate_id(&self) -> String {
        let mut rng = rand::thread_rng();
        let id: String = (0..6)
            .map(|_| {
                let idx = rng.gen_range(0..36);
                char::from_digit(idx, 36).unwrap()
            })
            .collect();
        id
    }
}

impl TaskStore for FsTaskStore {
    fn list(&self, filter: Filter) -> Result<Vec<Task>> {
        let tasks = self.all_tasks()?;
        Ok(match filter {
            Filter::All => tasks,
            Filter::Todo => tasks.into_iter().filter(|t| t.status == Status::Open).collect(),
            Filter::Done => tasks.into_iter().filter(|t| t.status == Status::Done).collect(),
        })
    }

    fn get(&self, id: &str) -> Result<Task> {
        let path = self.task_path(id);
        if !path.exists() {
            return Err(AppError::TaskNotFound(id.to_string()));
        }
        self.read_task(&path)
    }

    fn create(
        &mut self,
        title: &str,
        description: &str,
        due_date: Option<NaiveDate>,
    ) -> Result<Task> {
        let now = Utc::now();
        let task = Task {
            id: self.generate_id(),
            title: title.to_string(),
            description: description.to_string(),
            status: Status::Open,
            created_at: now,
            updated_at: now,
            due_date,
        };
        self.write_task(&task)?;
        Ok(task)
    }

    fn update(&mut self, id: &str, field: TaskField, value: &str) -> Result<Task> {
        let path = self.task_path(id);
        if !path.exists() {
            return Err(AppError::TaskNotFound(id.to_string()));
        }
        let mut task = self.read_task(&path)?;
        match field {
            TaskField::Title => task.title = value.to_string(),
            TaskField::Description => task.description = value.to_string(),
            TaskField::DueDate => {
                task.due_date = if value.is_empty() {
                    None
                } else {
                    Some(NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|e| {
                        AppError::ParseError(format!("Invalid date '{}': {}", value, e))
                    })?)
                };
            }
            TaskField::Status => {
                task.status = Status::from_str(value).ok_or_else(|| {
                    AppError::ParseError(format!(
                        "Invalid status '{}'. Use 'open' or 'done'",
                        value
                    ))
                })?;
            }
        }
        task.updated_at = Utc::now();
        self.write_task(&task)?;
        Ok(task)
    }

    fn delete(&mut self, id: &str) -> Result<()> {
        let path = self.task_path(id);
        if !path.exists() {
            return Err(AppError::TaskNotFound(id.to_string()));
        }
        fs::remove_file(path)?;
        Ok(())
    }

    fn clone_task(&mut self, id: &str) -> Result<Task> {
        let task = self.get(id)?;
        let now = Utc::now();
        let cloned = Task {
            id: self.generate_id(),
            title: format!("{} (copy)", task.title),
            description: task.description.clone(),
            status: Status::Open,
            created_at: now,
            updated_at: now,
            due_date: task.due_date,
        };
        self.write_task(&cloned)?;
        Ok(cloned)
    }

    fn migrate(&mut self, new_path: &Path) -> Result<()> {
        fs::create_dir_all(new_path)?;
        let entries = fs::read_dir(&self.data_dir)?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("md") {
                let dest = new_path.join(path.file_name().unwrap());
                if let Err(_) = fs::rename(&path, &dest) {
                    fs::copy(&path, &dest)?;
                    fs::remove_file(&path)?;
                }
            }
        }
        let _ = fs::remove_dir(&self.data_dir);
        self.data_dir = new_path.to_path_buf();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn setup() -> (FsTaskStore, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let store = FsTaskStore::new(dir.path().join("tasks")).unwrap();
        (store, dir)
    }

    #[test]
    fn test_create_and_list() {
        let (mut store, _dir) = setup();
        let task = store
            .create("Test Task", "Description here", None)
            .unwrap();
        assert_eq!(task.title, "Test Task");
        assert_eq!(task.status, Status::Open);

        let tasks = store.list(Filter::All).unwrap();
        assert_eq!(tasks.len(), 1);
    }

    #[test]
    fn test_filter() {
        let (mut store, _dir) = setup();
        store.create("Task 1", "", None).unwrap();
        let t2 = store.create("Task 2", "", None).unwrap();
        store
            .update(&t2.id, TaskField::Status, "done")
            .unwrap();

        assert_eq!(store.list(Filter::All).unwrap().len(), 2);
        assert_eq!(store.list(Filter::Todo).unwrap().len(), 1);
        assert_eq!(store.list(Filter::Done).unwrap().len(), 1);
    }

    #[test]
    fn test_get_and_delete() {
        let (mut store, _dir) = setup();
        let task = store.create("To Delete", "", None).unwrap();
        assert!(store.get(&task.id).is_ok());
        store.delete(&task.id).unwrap();
        assert!(store.get(&task.id).is_err());
    }

    #[test]
    fn test_clone() {
        let (mut store, _dir) = setup();
        let task = store
            .create("Original", "desc", NaiveDate::from_ymd_opt(2026, 5, 10))
            .unwrap();
        let cloned = store.clone_task(&task.id).unwrap();
        assert!(cloned.title.contains("Original"));
        assert_eq!(cloned.description, "desc");
        assert_eq!(cloned.due_date, task.due_date);
        assert_eq!(cloned.status, Status::Open);
        assert_ne!(cloned.id, task.id);
    }

    #[test]
    fn test_migrate() {
        let (mut store, _dir) = setup();
        store.create("Task", "", None).unwrap();
        let new_dir = tempdir().unwrap();
        let new_path = new_dir.path().join("new_tasks");

        store.migrate(&new_path).unwrap();
        assert_eq!(store.list(Filter::All).unwrap().len(), 1);
        assert!(new_path.join("tasks.md").exists() == false);
    }

    #[test]
    fn test_update_title() {
        let (mut store, _dir) = setup();
        let task = store.create("Old Title", "", None).unwrap();
        store
            .update(&task.id, TaskField::Title, "New Title")
            .unwrap();
        let updated = store.get(&task.id).unwrap();
        assert_eq!(updated.title, "New Title");
    }
}
