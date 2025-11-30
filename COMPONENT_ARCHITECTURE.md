# LexRain Component Architecture (V2)

## 🎯 Overview

LexRain now supports a **component-based architecture** inspired by React and Elm, providing better separation of concerns, reusability, and maintainability.

## 🚀 Usage

### Run with new architecture:
```bash
cargo run --release -- --v2
```

### Run with original architecture:
```bash
cargo run --release
```

## 📦 Component Structure

### Core Components

#### 1. **DashboardComponent** (`src/components/dashboard.rs`)
- **Responsibility**: Main screen showing statistics and navigation
- **State**:
  - Statistics (total, mastered, due)
  - Today's completed count
  - Completion message flag
- **Actions**: Navigate to other screens, quit

#### 2. **ReviewComponent** (`src/components/review.rs`)
- **Responsibility**: Flashcard review interface
- **State**:
  - Review queue
  - Current word
  - Review state (Question/Answer)
  - Progress tracking
- **Features**:
  - SM-2 spaced repetition algorithm
  - Progress bar
  - Quality rating (1-4)

#### 3. **DictionaryComponent** (`src/components/dictionary.rs`)
- **Responsibility**: Browse and search all words
- **State**:
  - Search input
  - Filtered word list
  - Selected index
- **Features**:
  - Real-time search
  - Learning status indicators (◯ New, ◐ Learning, ● Mastered)
  - Keyboard navigation

#### 4. **HistoryComponent** (`src/components/history.rs`)
- **Responsibility**: View recent review history
- **State**: Last 100 reviews
- **Features**:
  - Quality color coding
  - Timestamp display
  - Chinese definitions

#### 5. **StatisticsComponent** (`src/components/statistics.rs`)
- **Responsibility**: Visualize learning progress
- **State**:
  - Interval data (forgetting curve)
  - Daily review counts
- **Features**:
  - Forgetting curve chart
  - 30-day activity chart

### Reusable Components (`src/components/common/`)

#### **ProgressBar** (`progress_bar.rs`)
```rust
ProgressBar::new(current, total)
    .with_label("Custom label")
    .with_color(Color::Cyan)
    .render(frame, area);
```

#### **SearchInput** (`search_input.rs`)
```rust
let mut search = SearchInput::new()
    .with_placeholder("Type to search...");
search.handle_key(key);
search.render(frame, area);
```

#### **StatusBar** (`status_bar.rs`)
```rust
StatusBar::new()
    .add_item("q", "Quit")
    .add_item("h", "Help")
    .render(frame, area);
```

## 🏗️ Architecture Pattern

### Component Trait
```rust
pub trait Component {
    fn handle_key(&mut self, key: KeyEvent) -> Result<Action>;
    fn update(&mut self) -> Result<Action>;
    fn view(&mut self, frame: &mut Frame, area: Rect);
}
```

### Action System
```rust
pub enum Action {
    NavigateTo(Screen),
    Quit,
    None,
}
```

### Screen Management
```rust
pub enum Screen {
    Dashboard,
    Review,
    Dictionary,
    History,
    Statistics,
}
```

## 🎨 Benefits

### 1. **Separation of Concerns**
- Each component manages its own state
- Clear boundaries between UI and logic
- Easier to reason about code

### 2. **Reusability**
- Common components (ProgressBar, SearchInput, StatusBar)
- Can be used across different screens
- Consistent UI patterns

### 3. **Testability**
- Components can be tested in isolation
- Mock dependencies easily
- Unit test individual components

### 4. **Maintainability**
- Changes to one component don't affect others
- Easy to add new features
- Clear file structure

### 5. **Type Safety**
- Actions are strongly typed
- Compile-time guarantees
- No runtime surprises

## 📁 File Structure

```
src/
├── components/
│   ├── mod.rs              # Component trait & Action enum
│   ├── dashboard.rs        # Dashboard component
│   ├── review.rs           # Review component
│   ├── dictionary.rs       # Dictionary component
│   ├── history.rs          # History component
│   ├── statistics.rs       # Statistics component
│   └── common/
│       ├── mod.rs
│       ├── progress_bar.rs # Reusable progress bar
│       ├── search_input.rs # Reusable search input
│       └── status_bar.rs   # Reusable status bar
├── app_v2.rs               # Main app with component architecture
├── app.rs                  # Original app (legacy)
├── main.rs                 # Entry point
└── ...
```

## 🔄 Migration Path

### Original Architecture (app.rs)
- Monolithic App struct
- All state in one place
- UI rendering in separate ui.rs
- Event handling in main.rs

### New Architecture (app_v2.rs)
- Component-based structure
- State distributed across components
- Each component handles its own rendering
- Centralized action handling

## 🆕 New Features

### 1. **Improved Status Bar**
- Dynamic help text based on current screen
- Consistent across all screens
- Easy to customize

### 2. **Better Progress Tracking**
- Reusable ProgressBar component
- Customizable colors and labels
- Used in review screen

### 3. **Enhanced Search**
- Dedicated SearchInput component
- Placeholder text support
- Focus indication

### 4. **Cleaner Code**
- ~40% less code duplication
- Better organized
- Easier to extend

## 🎯 Future Enhancements

### Planned Features
1. **Popup/Modal Component** - For confirmations and alerts
2. **Form Component** - For adding/editing words
3. **Chart Component** - Better chart abstractions
4. **Theme System** - Customizable color schemes
5. **Plugin System** - Extensible architecture

### Third-Party Widgets to Consider
- `tui-input` - Advanced text input
- `tui-textarea` - Multi-line text editing
- `tui-tree-widget` - Hierarchical data display
- `ratatui-explorer` - File browser integration

## 📊 Comparison

| Feature | Original (app.rs) | Component-based (app_v2.rs) |
|---------|-------------------|------------------------------|
| Lines of code | ~500 | ~800 (with reusable components) |
| Testability | Medium | High |
| Reusability | Low | High |
| Maintainability | Medium | High |
| Extensibility | Medium | High |
| Learning curve | Low | Medium |

## 🔧 Development Tips

### Adding a New Component
1. Create file in `src/components/`
2. Implement `Component` trait
3. Add to `mod.rs`
4. Integrate in `app_v2.rs`

### Adding a New Screen
1. Add variant to `Screen` enum
2. Create component
3. Add navigation logic in `AppV2::navigate_to()`
4. Add rendering in `AppV2::render()`
5. Add status bar in `AppV2::render_footer()`

### Creating Reusable Components
1. Place in `src/components/common/`
2. Make stateless if possible
3. Use builder pattern for configuration
4. Provide `render()` method

## 📚 Resources

- [Ratatui Documentation](https://ratatui.rs/)
- [Component Architecture Pattern](https://ratatui.rs/concepts/application-patterns/component-architecture/)
- [Third-Party Widgets](https://ratatui.rs/showcase/third-party-widgets/)

## 🎉 Conclusion

The component-based architecture provides a solid foundation for future development. It's more maintainable, testable, and extensible than the original monolithic approach.

**Try it out:**
```bash
cargo run --release -- --v2
```
