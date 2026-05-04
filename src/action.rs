use chrono::NaiveDate;

use crate::task::model::Filter;

#[derive(Debug, Clone)]
pub enum Action {
    MoveUp,
    MoveDown,

    NewTask {
        title: String,
        description: String,
        due: Option<NaiveDate>,
    },
    NewTaskPrefill,
    EditTaskPrefill,
    OpenTaskExternal(String),
    EditTask {
        id: String,
        fields: Vec<(String, String)>,
    },
    DeleteTask(String),
    ShowConfirmDelete(String),
    ToggleDoneSelected,
    ShowDetailSelected,
    DeleteSelected,
    CloneTask(String),

    SetFilter(Filter),
    ShowDetail(usize),
    CloseModal,

    ToggleCommandBar,
    OpenConfig,
    ConfigAction {
        key: Option<String>,
        value: Option<String>,
    },
    MigrateData(String),
    Reload,

    SetStatusMessage(String),

    Quit,
}
