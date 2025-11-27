# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

LexRain is a terminal-based vocabulary learning application written in Rust that uses the SuperMemo-2 (SM-2) spaced repetition algorithm. The app stores learning progress locally in SQLite and provides a TUI interface built with ratatui.

## Development Commands

### Build and Run
```bash
# Build in release mode
cargo build --release

# Run the application
cargo run --release

# Import vocabulary from JSON file
cargo run --release -- --import sample_words.json
```

### Testing and Linting
```bash
# Run tests
cargo test

# Check code without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy
```

## Architecture Overview

### Module Structure

The codebase follows a clean separation of concerns across 8 modules:

- **main.rs**: Entry point, event loop, and CLI argument handling (import functionality)
- **app.rs**: Central state management - tracks current screen, review queue, and statistics
- **db.rs**: SQLite abstraction layer for `words` and `learning_log` tables
- **models.rs**: Core data structures (`Word`, `LearningLog`, `LearningStatus`)
- **sm2.rs**: SuperMemo-2 algorithm implementation with `update_memory_state()` and `process_review()`
- **ui.rs**: All ratatui rendering logic for Dashboard, Review, and Dictionary screens
- **event.rs**: Keyboard event handling and tick system
- **tui.rs**: Terminal initialization and cleanup utilities

### Data Flow

```
User Input → EventHandler → Main Loop → App State Update
                                            ↓
                            ┌───────────────┼───────────────┐
                            ↓               ↓               ↓
                        Database         SM-2 Algo      UI Render
                        (db.rs)          (sm2.rs)       (ui.rs)
```

### Database Schema

**words table**: Stores vocabulary entries
- `id` (PK), `spelling` (UNIQUE), `phonetic`, `definition`, `tags`

**learning_log table**: Tracks SM-2 algorithm state per word
- `word_id` (FK), `repetition`, `interval`, `e_factor`, `next_review`, `status`

Status codes: 0=New, 1=Learning, 2=Mastered (interval > 21 days)

### SuperMemo-2 Implementation

The SM-2 algorithm in `sm2.rs` maps user ratings (1-4) to quality scores:
- **1 (Forgot)** / **2 (Hard)**: quality < 3 → resets repetition to 0, interval to 1
- **3 (Good)** / **4 (Easy)**: quality >= 3 → increases interval exponentially

Interval calculation:
- First review: 1 day
- Second review: 6 days
- Subsequent: `interval = previous_interval × e_factor`

E-Factor update: `EF' = EF + (0.1 - (5 - quality) × (0.08 + (5 - quality) × 0.02))` with minimum 1.3

## Application Screens

### CurrentScreen Enum
- **Dashboard**: Main screen showing statistics (total, mastered, due today) and mastery progress bar
- **Review**: Card-based review interface with Question/Answer states
- **Dictionary**: Placeholder for future search functionality
- **Exiting**: Terminal cleanup state

### Review Flow
1. User presses 'r' on Dashboard → `App::start_review()` loads due words from DB
2. Queue is reversed (using `Vec::pop()`) to process oldest reviews first
3. Question state shows spelling and phonetic, waits for Space/Enter
4. Answer state shows definition, waits for rating (1-4)
5. Rating triggers `sm2::process_review()` → updates DB → advances to next card
6. When queue empties, auto-returns to Dashboard and refreshes stats

## Code Conventions

### Error Handling
All fallible operations return `anyhow::Result<T>`. Database operations propagate errors with `?` operator. Stats are gracefully degraded with `.unwrap_or((0, 0, 0))`.

### Time Handling
All timestamps use `chrono::DateTime<Utc>` and are serialized to RFC3339 strings in SQLite. The `next_review` field determines when a word appears in the review queue.

### State Management
`App` struct owns the `Database` connection and maintains:
- `review_queue: Vec<(Word, LearningLog)>` - populated by `get_due_reviews()`
- `current_review_item: Option<(Word, LearningLog)>` - the active card
- `stats: (i64, i64, i64)` - cached counts refreshed after reviews and screen changes

## JSON Import Format

Words are imported from JSON arrays with this structure:
```json
[
  {
    "spelling": "algorithm",
    "phonetic": "ˈælɡərɪðəm",
    "definition": "n. A step-by-step procedure...",
    "tags": "programming,computer-science"
  }
]
```

Import uses `INSERT OR IGNORE` to prevent duplicates based on `spelling` uniqueness. Each imported word automatically gets a `learning_log` entry initialized with:
- `repetition: 0`, `interval: 0`, `e_factor: 2.5`, `next_review: now()`, `status: 0`

## Key Dependencies

- **ratatui 0.29**: TUI framework (widgets, layout, rendering)
- **crossterm 0.28**: Cross-platform terminal control (keyboard events, raw mode)
- **rusqlite 0.32**: SQLite bindings with bundled library
- **chrono 0.4**: DateTime handling with serde support
- **clap 4.5**: CLI argument parsing with derive macros
- **serde/serde_json 1.0**: JSON serialization for word import

## Database Location

The SQLite database is created at `lexrain.db` in the current working directory. To reset all progress, delete this file and re-import vocabulary.
