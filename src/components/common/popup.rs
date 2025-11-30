use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

/// 通用浮窗组件，支持滚动和关闭
pub struct Popup {
    scroll: u16,
    title: String,
}

impl Popup {
    pub fn new(title: String) -> Self {
        Self {
            scroll: 0,
            title,
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub fn reset_scroll(&mut self) {
        self.scroll = 0;
    }

    /// 渲染浮窗，返回内容区域
    pub fn render(&mut self, frame: &mut Frame, area: Rect, content_lines: Vec<Line<'_>>) {
        // 计算居中的浮窗区域（80% 宽度，90% 高度）
        let popup_area = centered_rect(80, 90, area);

        // 清除背景
        frame.render_widget(Clear, popup_area);

        // 渲染浮窗边框
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", self.title))
            .title_bottom(" q: 关闭 | j/k: 滚动 ")
            .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

        let inner_area = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        // 渲染内容
        let content_height = content_lines.len() as u16;
        let paragraph = Paragraph::new(content_lines)
            .wrap(Wrap { trim: true })
            .scroll((self.scroll, 0));
        frame.render_widget(paragraph, inner_area);

        // 渲染滚动条
        if content_height > inner_area.height {
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓")),
                inner_area.inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut ScrollbarState::new(content_height as usize)
                    .position(self.scroll as usize),
            );
        }
    }
}

/// 计算居中的矩形区域
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
