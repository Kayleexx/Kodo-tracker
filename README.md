# Kodo: Track Your Development Activities

Kodo is a **developer activity tracker** with a clean terminal UI, built in Rust using [ratatui](https://github.com/ratatui-org/ratatui).  
It helps you log activities, track time, and even sync your **Git commits** into the dashboard, so you get real stats about your coding sessions without extra effort.

---

## Features

- 📋 **Activities table** — log tasks you’re working on.
- ⏱ **Duration tracking** — keep track of how much time you spend.
- 📊 **Stats view** — toggle a bar chart of activity durations.
- 🌱 **Git integration** — automatically sync commit history from the current repo as activities.
- 💾 **Persistent storage** — saves activities to a JSON file on disk.
- 🖥 **TUI navigation** — built with `ratatui` for a fast, keyboard-driven UI.

---

## Demo

(Insert a GIF or screenshot here once you record one)

---

## Installation

### From source
Make sure you have [Rust](https://www.rust-lang.org/tools/install) installed.

```bash
git clone https://github.com/Kayleexx/Kodo-tracker.git
cd kodo
cargo install --path .
````

This will install the `kodo` binary globally.

---

## Usage

Run inside any Git repository to also sync commits:

```bash
kodo
```

### Keyboard shortcuts

| Key | Action                        |
| --- | ----------------------------- |
| `q` | Quit                          |
| `a` | Add a new activity            |
| `d` | Delete selected activity      |
| `f` | Filter activities by duration |
| `r` | Reset filters                 |
| `s` | Sort activities               |
| `v` | Toggle stats view             |
| `g` | Sync Git commits              |

---

## 🗂 Data Storage

Kodo stores activities in a JSON file (by default in your project directory).
Each activity has:

```json
{
  "id": 1,
  "name": "Fix login bug",
  "duration_minutes": 45,
  "date": "2025-09-04"
}
```

---

## 🦀 Tech Stack

* [ratatui](https://github.com/ratatui-org/ratatui) — Terminal UI
* [git2](https://crates.io/crates/git2) — Git integration
* [anyhow](https://crates.io/crates/anyhow) — Error handling
* [serde](https://crates.io/crates/serde) + [serde\_json](https://crates.io/crates/serde_json) — Persistence


---

## 📜 License

MIT License © 2025 [Mitali](https://github.com/Kayleexx)

---

https://github.com/user-attachments/assets/7c99b9ba-513a-435a-bf56-2436b649157b



