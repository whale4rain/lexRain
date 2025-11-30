# 主题使用指南

## 概述

项目使用统一的主题系统（`src/theme.rs`），提供蓝红白配色方案和标准化的边框样式。

## 配色方案

参考 [ratatui 官方示例](https://ratatui.rs/examples/widgets/block/)，采用以下配色：

| 颜色 | 用途 | 常量 |
|------|------|------|
| **蓝色 (Cyan)** | 主色调，标题，重点信息 | `Theme::PRIMARY` |
| **品红 (Magenta)** | 强调色，重要提示 | `Theme::ACCENT` |
| **绿色 (Green)** | 成功状态 | `Theme::SUCCESS` |
| **黄色 (Yellow)** | 警告/快捷键 | `Theme::WARNING` |
| **白色 (White)** | 主要文本 | `Theme::FOREGROUND` |
| **灰色 (DarkGray)** | 次要文本 | `Theme::SECONDARY` |

## 边框样式

### 标准边框（蓝色粗边框）
```rust
// 无标题
let block = Theme::block_default();

// 带标题
let block = Theme::block_with_title(" 📊 Learning Stats ");
```

### 强调边框（品红色粗边框）
```rust
let block = Theme::block_accent();
let block = Theme::block_accent_with_title(" Today ");
```

### 成功边框（绿色粗边框）
```rust
let block = Theme::block_success();
let block = Theme::block_success_with_title(" 🎉 Actions ");
```

### 警告边框（黄色粗边框）
```rust
let block = Theme::block_warning();
let block = Theme::block_warning_with_title(" ⚠️ Warning ");
```

## 文本样式

### 预定义样式
```rust
// 标题文本（蓝色粗体）
Span::styled("Title", Theme::text_title())

// 强调文本（品红色粗体）
Span::styled("Important", Theme::text_accent())

// 成功文本（绿色粗体）
Span::styled("Success", Theme::text_success())

// 警告文本（黄色粗体）
Span::styled("Warning", Theme::text_warning())

// 普通文本（白色）
Span::styled("Normal", Theme::text_normal())

// 次要文本（灰色）
Span::styled("Secondary", Theme::text_secondary())
```

## Dashboard 示例

```rust
use crate::theme::Theme;

// Stats 卡片 - 蓝色边框
let stats_widget = Paragraph::new(lines)
    .block(Theme::block_with_title(" 📊 Learning Stats "))
    .style(Theme::text_normal());

// Progress 卡片 - 蓝色边框
let progress_widget = Paragraph::new(text)
    .block(Theme::block_with_title(" 📅 Today's Progress "))
    .style(Theme::text_normal());

// 完成消息 - 绿色边框
let completion_msg = Paragraph::new(lines)
    .block(Theme::block_success_with_title(" 🎉 Actions "));

// Today 日期 - 品红色边框（强调）
let today_widget = Paragraph::new(text)
    .block(Theme::block_accent_with_title(" Today "))
    .style(Theme::text_title());

// 进度条 - 绿色边框
let progress_title = format!(" ✓ Mastered: {} / {} ", mastered, total);
let gauge = Gauge::default()
    .block(Theme::block_success().title(progress_title))
    .gauge_style(Theme::text_success());
```

## 修改配色

如需修改全局配色，编辑 `src/theme.rs`：

```rust
impl Theme {
    // 修改这些常量即可全局生效
    pub const PRIMARY: Color = Color::Cyan;      // 改为你喜欢的颜色
    pub const ACCENT: Color = Color::Magenta;
    pub const SUCCESS: Color = Color::Green;
    pub const WARNING: Color = Color::Yellow;
    pub const FOREGROUND: Color = Color::White;
    pub const SECONDARY: Color = Color::DarkGray;
}
```

支持的颜色：
- `Color::Cyan`, `Color::Magenta`, `Color::Green`, `Color::Yellow`
- `Color::Red`, `Color::Blue`, `Color::White`, `Color::Black`
- `Color::DarkGray`, `Color::Gray`, `Color::LightBlue`, etc.
- `Color::Rgb(r, g, b)` - 自定义 RGB

## 其他组件应用

### Review 组件
```rust
let block = Theme::block_with_title(" 📖 Review ");
let focused_block = Theme::block_accent_with_title(" 📖 Review [FOCUSED] ");
```

### Dictionary 组件
```rust
let search_block = if searching {
    Theme::block_warning_with_title(" 🔍 Searching... ")
} else {
    Theme::block_with_title(" 🔍 Search ")
};
```

### History 组件
```rust
let block = Theme::block_with_title(" 📜 History ");
let selected_style = Theme::text_highlight();  // 反转高亮
```

## 最佳实践

1. **统一使用主题**：所有边框和文本样式都通过 `Theme` 获取
2. **语义化颜色**：
   - 蓝色：主要信息、标题
   - 品红：强调、当前焦点
   - 绿色：成功状态、完成
   - 黄色：警告、快捷键提示
3. **边框类型**：默认使用 `BorderType::Thick`（粗边框）
4. **避免硬编码颜色**：不要直接使用 `Color::Cyan`，而是使用 `Theme::PRIMARY`

## 视觉效果

```
╔═══════════════════════════════════╗  <- Thick 蓝色边框
║ 📊 Learning Stats                 ║  <- 蓝色粗体标题
║                                   ║
║ 📚 Learning: 150 words            ║  <- 白色文本 + 蓝色粗体数字
║                                   ║
║ ✓ Mastered: 75 words              ║  <- 绿色图标和数字
║                                   ║
║ ⏰ Due Now: 25 words               ║  <- 黄色图标和数字
║                                   ║
╚═══════════════════════════════════╝
```

## 参考

- [Ratatui Block Examples](https://ratatui.rs/examples/widgets/block/)
- [Ratatui Color Reference](https://docs.rs/ratatui/latest/ratatui/style/enum.Color.html)
