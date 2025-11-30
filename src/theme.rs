use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders},
};

/// 应用主题配色方案
/// 参考 ratatui 官方示例的蓝红白配色
pub struct Theme;

impl Theme {
    // === 主要颜色 ===
    /// 主色调 - 蓝色（用于标题、重点信息）
    pub const PRIMARY: Color = Color::Blue;
    
    /// 强调色 - 红色/品红（用于重要提示、警告）
    pub const ACCENT: Color = Color::Red;
    
    /// 成功色 - 绿色
    pub const SUCCESS: Color = Color::Green;
    
    /// 警告色 - 黄色
    pub const WARNING: Color = Color::Yellow;
    
    /// 前景色 - 白色（主要文本）
    pub const FOREGROUND: Color = Color::White;
    
    /// 次要文本 - 灰色
    pub const SECONDARY: Color = Color::DarkGray;

    // === 边框样式 ===
    
    /// 标准边框样式（白色粗边框）
    pub fn block_default() -> Block<'static> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(Self::FOREGROUND))
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

    /// 强调边框样式（白色粗边框）
    pub fn block_accent() -> Block<'static> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(Self::FOREGROUND))
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

    /// 成功边框样式（白色粗边框）
    pub fn block_success() -> Block<'static> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(Self::FOREGROUND))
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

    /// 警告边框样式（白色粗边框）
    pub fn block_warning() -> Block<'static> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(Self::FOREGROUND))
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
