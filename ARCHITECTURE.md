# LexRain Architecture - Component-Based Design

## Overview
LexRain uses a component-based architecture inspired by React and Elm, where each screen is a self-contained component with its own state and event handling.

## Component Trait
```rust
pub trait Component {
    type Message;

    fn update(&mut self, message: Self::Message) -> anyhow::Result<Option<Action>>;
    fn view(&mut self, frame: &mut Frame, area: Rect);
    fn handle_key(&mut self, key: KeyEvent) -> Option<Self::Message>;
}
```

## Component Structure

### 1. Dashboard Component
- **State**: Statistics, today's completed count, completion message
- **Messages**: StartReview, StartLearnNew, EnterDictionary, EnterHistory, EnterStatistics
- **Actions**: Navigate to other screens

### 2. Review Component
- **State**: Current word, review queue, progress, review state (Question/Answer)
- **Messages**: ShowAnswer, SubmitReview(quality), NextCard
- **Actions**: Complete review session, return to dashboard

### 3. Dictionary Component
- **State**: Search input, word list, selected index
- **Messages**: UpdateSearch, SelectNext, SelectPrevious
- **Actions**: None (self-contained)

### 4. History Component
- **State**: Review history list
- **Messages**: None (read-only)
- **Actions**: None

### 5. Statistics Component
- **State**: Interval data, daily data
- **Messages**: None (read-only)
- **Actions**: None

## App State Machine
```
Dashboard <-> Review
    |
    +---> Dictionary
    |
    +---> History
    |
    +---> Statistics
```

## Benefits
1. **Separation of Concerns**: Each component manages its own state
2. **Reusability**: Components can be reused in different contexts
3. **Testability**: Components can be tested in isolation
4. **Maintainability**: Changes to one component don't affect others
5. **Type Safety**: Messages are strongly typed per component

## New Features to Add
1. **Search Component**: Reusable search input with autocomplete
2. **Progress Bar Component**: Reusable progress indicator
3. **Popup/Modal Component**: For confirmations and alerts
4. **Input Form Component**: For adding/editing words
5. **Chart Component**: Wrapper for statistics charts
