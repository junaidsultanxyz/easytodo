# EasyTodo

A terminal-based todo app built in Rust. Minimal, keyboard-driven, with file-based storage and an MCP server for AI integration.
<img width="986" height="504" alt="image" src="https://github.com/user-attachments/assets/43c414ad-620c-42a5-bed4-44c0bc5a3f78" />

## Installation

### From source

```bash
# Clone and install to /usr/local/bin
git clone https://github.com/junaidxyz/easytodo
cd easytodo
make install

# Or install locally
make install-local  # ~/.local/bin/easytodo
```

### Via cargo

```bash
cargo install easytodo
```

### Via release binary

Download the latest binary from [Releases](https://github.com/junaidxyz/easytodo/releases), then:

```bash
chmod +x easytodo
sudo mv easytodo /usr/local/bin/easytodo
```

## Usage

Run `easytodo` in your terminal.

### Navigation

| Key | Action |
|---|---|
| `j` / `k` | Move selection up/down |
| `Enter` / `Space` | Toggle task done/undone |
| `l` / `o` | Open task detail view |
| `1` | Show all tasks |
| `2` | Show only pending tasks |
| `3` | Show only done tasks |

### Commands

Press **Ctrl+P** to open the command bar. Commands use quoted strings.

| Command | Example |
|---|---|
| `new "Buy milk"` | Create a task |
| `new "Buy milk" "Grocery store"` | Create with description |
| `new "Buy milk" due:2026-06-01` | Create with due date |
| `new "Buy milk" "Get from store" due:2026-06-01` | Full example |
| `edit . title:"New title"` | Edit current task's title |
| `edit . desc:"Updated description"` | Edit current task's description |
| `edit . due:2026-07-01` | Edit current task's due date |
| `done .` | Mark current task done |
| `undone .` | Mark current task undone |
| `delete .` | Delete current task |
| `clone .` | Duplicate current task |
| `open .` | Open current task in editor |
| `list` | Show all tasks in terminal |
| `config` | Open config file in editor |
| `migrate ~/Documents` | Move task storage directory |
| `reload` | Reload all tasks from disk |
| `help` | Show command reference and shortcuts |
| `quit` | Exit EasyTodo |

> `.` resolves to the currently highlighted task.

### Other shortcuts

| Key | Action |
|---|---|
| `Ctrl+N` | Quick-create a new task |
| `Ctrl+E` | Edit the selected task |
| `Ctrl+D` | Delete the selected task |
| `Ctrl+B` | Open config file |
| `Ctrl+H` | Show help panel |
| `Ctrl+R` | Reload config and tasks |
| `Ctrl+Q` | Quit |
| `Esc` | Close command bar / go back |

## Data Storage

Tasks are stored as individual `.md` files with YAML frontmatter in `~/easytodo/` (configurable via `migrate` command). Each file looks like:

```yaml
---
title: Buy milk
description: From the grocery store
status: pending
due_date: 2026-06-01
created_at: 2026-05-01T10:00:00Z
updated_at: 2026-05-01T10:00:00Z
---
```

## MCP Server

EasyTodo includes an MCP server for AI integration (Claude Code, Cursor, OpenCode, etc.).

```bash
# Start the MCP server
easytodo-mcp
```

### Client configuration

**OpenCode** — add to `.config/opencode.json`:
```json
{
  "mcp": {
    "easytodo": {
      "type": "command",
      "command": "easytodo-mcp"
    }
  }
}
```

**Claude Code**:
```bash
claude mcp add easytodo -s command -- easytodo-mcp
```

**Cursor** — add to `.cursor/mcp.json`:
```json
{
  "mcpServers": {
    "easytodo": {
      "command": "easytodo-mcp"
    }
  }
}
```

MCP tools available: `list_tasks`, `get_task`, `create_task`, `update_task`, `delete_task`, `done_task`, `undone_task`, `help`, `get_config`, `set_config`.

## Configuration

Config file at `~/.config/easytodo/config.toml` (see [`easytodo.example.toml`](easytodo.example.toml) for defaults):

```toml
data_dir = "/home/user/easytodo"
editor = "nvim"

[theme]
selected_bg = "rgb(60,60,80)"
done_fg = "rgb(100,140,100)"
border = "rgb(80,80,120)"
command_bar_bg = "rgb(30,30,50)"
modal_bg = "rgb(25,25,45)"
title_fg = "rgb(180,180,220)"
normal_bg = "rgb(20,20,35)"
status_bar_fg = "rgb(130,130,160)"

[keybindings]
move_down = "j"
move_up = "k"
toggle_done = "Enter"
show_detail = "l"
filter_all = "1"
filter_todo = "2"
filter_done = "3"
new_task = "Ctrl+N"
edit_task = "Ctrl+E"
delete_task = "Ctrl+D"
open_config = "Ctrl+B"
command_bar = "Ctrl+P"
help = "Ctrl+H"
reload = "Ctrl+R"
quit = "Ctrl+Q"
```

Colors support `rgb(R,G,B)` and `#RRGGBB` formats. Keybindings support single keys (`j`, `Enter`, `Space`, `Esc`, `Up`, `Down`) and `Ctrl+` prefix for control combinations.

## License

MIT
