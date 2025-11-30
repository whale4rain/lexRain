use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders},
};

/// 应用主题配色方案
/// 
/// ## 背景色配置
/// 
/// 所有组件底层都会应用 `BACKGROUND` 常量定义的背景色。
/// 
/// ### 使用透明背景：
/// ```rust
/// pub const BACKGROUND: Color = Color::Reset;
/// ```
/// 
/// ### 使用深色背景（当前配置）：
/// ```rust
/// pub const BACKGROUND: Color = Color::Rgb(30, 30, 40); // 深蓝灰色
/// ```
/// 
/// ### 其他背景色选项：
/// ```rust
/// pub const BACKGROUND: Color = Color::Black;           // 纯黑色
/// pub const BACKGROUND: Color = Color::Rgb(20, 20, 20); // 深灰色
/// pub const BACKGROUND: Color = Color::Rgb(25, 35, 45); // 深蓝色
/// ```
/// 
/// 修改 `BACKGROUND` 常量后重新编译即可生效。
pub struct Theme;
#[allow(unused)]

impl Theme {
    // === 主要颜色 ===
    /// 主色调 - 蓝色（用于标题、重点信息）
    pub const PRIMARY: Color = Color::Rgb(156, 198, 219);
    
    /// 强调色 - 红色（用于重要提示、焦点）
    pub const ACCENT: Color = Color::Rgb(207, 75, 0);
    
    /// 成功色 - 绿色
    pub const SUCCESS: Color = Color::Green;
    
    /// 警告色 - 黄色
    pub const WARNING: Color = Color::Rgb(221, 186, 125);
    
    /// 信息色 - 粉色
    pub const INFO: Color = Color::Rgb(207, 103, 155); // Pink/HotPink
    
    /// 前景色 - 白色（主要文本）
    pub const FOREGROUND: Color = Color::Rgb(252, 246, 217);
    
    /// 次要文本 - 灰色
    pub const SECONDARY: Color = Color::DarkGray;
    
    /// 背景色 - 深色背景（可配置为透明或有颜色）
    /// 使用 Color::Reset 表示透明背景
    /// 使用其他颜色值表示有颜色的背景
    ///pub const BACKGROUND: Color = Color::Rgb(25, 25, 35); // 深蓝灰色背景
    pub const BACKGROUND: Color = Color::Reset; // 如需透明背景，取消注释此行并注释上一行

    // === 边框样式 ===
    
    /// 标准边框样式（白色粗边框 + 背景色）
    pub fn block_default() -> Block<'static> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(Self::FOREGROUND))
            .style(Style::default().bg(Self::BACKGROUND))
    }

    /// 带标题的标准边框（白色边框，蓝底蓝色文字标题）
    pub fn block_with_title(title: &'static str) -> Block<'static> {
        Self::block_default()
            .title(title)
            .title_style(Style::default()
                .fg(Self::PRIMARY)
                .bg(Self::FOREGROUND)
                .add_modifier(Modifier::BOLD))
    }

    /// 强调边框样式（白色粗边框 + 背景色）
    pub fn block_accent() -> Block<'static> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(Self::FOREGROUND))
            .style(Style::default().bg(Self::BACKGROUND))
    }

    /// 带标题的强调边框（白色边框，白底红色文字标题）
    pub fn block_accent_with_title(title: &'static str) -> Block<'static> {
        Self::block_accent()
            .title(title)
            .title_style(Style::default()
                .fg(Self::ACCENT)
                .bg(Self::FOREGROUND)
                .add_modifier(Modifier::BOLD))
    }

    /// 成功边框样式（白色粗边框 + 背景色）
    pub fn block_success() -> Block<'static> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(Self::FOREGROUND))
            .style(Style::default().bg(Self::BACKGROUND))
    }

    /// 带标题的成功边框（白色边框，白底绿色文字标题）
    pub fn block_success_with_title(title: &'static str) -> Block<'static> {
        Self::block_success()
            .title(title)
            .title_style(Style::default()
                .fg(Self::SUCCESS)
                .bg(Self::FOREGROUND)
                .add_modifier(Modifier::BOLD))
    }

    /// 警告边框样式（白色粗边框 + 背景色）
    pub fn block_warning() -> Block<'static> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(Self::FOREGROUND))
            .style(Style::default().bg(Self::BACKGROUND))
    }

    /// 带标题的警告边框（白色边框，白底黄色文字标题）
    pub fn block_warning_with_title(title: &'static str) -> Block<'static> {
        Self::block_warning()
            .title(title)
            .title_style(Style::default()
                .fg(Self::WARNING)
                .bg(Self::FOREGROUND)
                .add_modifier(Modifier::BOLD))
    }

    /// 双重边框样式（白色双重边框）
    pub fn block_double() -> Block<'static> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Self::FOREGROUND))
    }

    /// 圆角边框样式（白色圆角边框）
    pub fn block_rounded() -> Block<'static> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Self::FOREGROUND))
    }

    // === 文本样式 ===

    /// 标题文本样式（蓝色粗体）
    pub fn text_title() -> Style {
        Style::default().fg(Self::PRIMARY).add_modifier(Modifier::BOLD)
    }

    /// 强调文本样式（品红色粗体）
    pub fn text_accent() -> Style {
        Style::default().fg(Self::ACCENT).add_modifier(Modifier::BOLD)
    }

    /// 成功文本样式（绿色粗体）
    pub fn text_success() -> Style {
        Style::default().fg(Self::SUCCESS).add_modifier(Modifier::BOLD)
    }

    /// 警告文本样式（黄色粗体）
    pub fn text_warning() -> Style {
        Style::default().fg(Self::WARNING).add_modifier(Modifier::BOLD)
    }

    /// 信息文本样式（橙色粗体）
    pub fn text_info() -> Style {
        Style::default().fg(Self::INFO).add_modifier(Modifier::BOLD)
    }

    /// 普通文本样式（白色）
    pub fn text_normal() -> Style {
        Style::default().fg(Self::FOREGROUND)
    }

    /// 次要文本样式（灰色）
    pub fn text_secondary() -> Style {
        Style::default().fg(Self::SECONDARY)
    }

    /// 高亮文本样式（蓝色 + 反转）
    pub fn text_highlight() -> Style {
        Style::default()
            .fg(Self::PRIMARY)
            .add_modifier(Modifier::REVERSED)
    }
}
