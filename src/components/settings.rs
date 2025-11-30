use super::{Action, Component, Screen};
use crate::db::Database;
use crate::theme::Theme;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

pub struct SettingsComponent {
    db: Database,
    daily_goal: i64,
    editing: bool,
    input_buffer: String,
    message: Option<String>,
    scroll: u16,  // 滚动位置
}

impl SettingsComponent {
    pub fn new(db: Database) -> Result<Self> {
        let daily_goal = db.get_daily_goal()?;
        Ok(Self {
            db,
            daily_goal,
            editing: false,
            input_buffer: String::new(),
            message: None,
            scroll: 0,
        })
    }

    fn start_editing(&mut self) {
        self.editing = true;
        self.input_buffer = self.daily_goal.to_string();
        self.message = None;
    }

    fn cancel_editing(&mut self) {
        self.editing = false;
        self.input_buffer.clear();
        self.message = None;
    }

    fn save_setting(&mut self) -> Result<()> {
        if let Ok(goal) = self.input_buffer.parse::<i64>() {
            if goal > 0 && goal <= 1000 {
                self.db.set_daily_goal(goal)?;
                self.daily_goal = goal;
                self.editing = false;
                self.input_buffer.clear();
                self.message = Some("✓ Settings saved successfully!".to_string());
                Ok(())
            } else {
                self.message = Some("Error: Goal must be between 1 and 1000".to_string());
                Ok(())
            }
        } else {
            self.message = Some("Error: Invalid number".to_string());
            Ok(())
        }
    }
}

impl Component for SettingsComponent {
    fn handle_key(&mut self, key: KeyEvent) -> Result<Action> {
        if self.editing {
            match key.code {
                KeyCode::Esc => {
                    self.cancel_editing();
                    Ok(Action::None)
                }
                KeyCode::Enter => {
                    self.save_setting()?;
                    Ok(Action::None)
                }
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    if self.input_buffer.len() < 4 {
                        self.input_buffer.push(c);
                    }
                    Ok(Action::None)
                }
                KeyCode::Backspace => {
                    self.input_buffer.pop();
                    Ok(Action::None)
                }
                _ => Ok(Action::None),
            }
        } else {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => Ok(Action::NavigateTo(Screen::Dashboard)),
                KeyCode::Char('e') | KeyCode::Enter => {
                    self.start_editing();
                    Ok(Action::None)
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.scroll = self.scroll.saturating_add(1);
                    Ok(Action::None)
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.scroll = self.scroll.saturating_sub(1);
                    Ok(Action::None)
                }
                _ => Ok(Action::None),
            }
        }
    }

    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10), // Daily goal setting
                Constraint::Length(3),  // Message
                Constraint::Min(10),    // Help & Rules with scroll
            ])
            .margin(2)
            .split(area);

        // Daily goal setting
        let goal_lines = if self.editing {
            vec![
                Line::from(vec![
                    Span::styled("📊 ", Theme::text_warning()),
                    Span::styled(
                        "每日复习目标",
                        Theme::text_title(),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("输入目标 (1-1000): "),
                    Span::styled(
                        &self.input_buffer,
                        Theme::text_warning()
                            .add_modifier(Modifier::UNDERLINED),
                    ),
                    Span::styled("_", Theme::text_warning()),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Enter", Theme::text_success()),
                    Span::raw(" 保存 | "),
                    Span::styled("Esc", Theme::text_accent()),
                    Span::raw(" 取消"),
                ]),
            ]
        } else {
            vec![
                Line::from(vec![
                    Span::styled("📊 ", Theme::text_warning()),
                    Span::styled(
                        "每日复习目标",
                        Theme::text_title(),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("当前目标: "),
                    Span::styled(
                        format!("{} 个/天", self.daily_goal),
                        Theme::text_title(),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("按 "),
                    Span::styled(
                        "'e'",
                        Theme::text_warning(),
                    ),
                    Span::raw(" 编辑"),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("💡 ", Theme::text_secondary()),
                    Span::styled("完成每日目标可在日历上获得打卡标记", Theme::text_secondary()),
                ]),
            ]
        };

        let goal_widget = Paragraph::new(goal_lines)
            .block(Theme::block_with_title(" ⚙️  设置 "));
        frame.render_widget(goal_widget, chunks[0]);

        // Message
        if let Some(msg) = &self.message {
            let msg_style = if msg.starts_with("✓") {
                Theme::text_success()
            } else {
                Theme::text_accent()
            };
            let message_widget = Paragraph::new(msg.as_str())
                .style(msg_style)
                .block(Theme::block_default());
            frame.render_widget(message_widget, chunks[1]);
        } else {
            let placeholder = Paragraph::new("")
                .block(Theme::block_default());
            frame.render_widget(placeholder, chunks[1]);
        }

        // Help & Learning Rules (scrollable)
        let help_lines = vec![
            Line::from(vec![
                Span::styled("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", Theme::text_secondary()),
            ]),
            Line::from(vec![
                Span::styled("📚 学习规则说明", Theme::text_title()),
            ]),
            Line::from(vec![
                Span::styled("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", Theme::text_secondary()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("● 词汇库 vs 单词本", Theme::text_warning()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("词汇库", Theme::text_title()),
                Span::raw("（Dashboard 显示）"),
            ]),
            Line::from("  • 所有已学习的单词，包括各种学习状态"),
            Line::from("  • 学习中、已掌握等状态的单词都在这里"),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("单词本", Theme::text_title()),
                Span::raw("（Wordbook 页面）"),
            ]),
            Line::from("  • ECDICT词典预定义的分类标签"),
            Line::from("  • 如：中考、高考、CET-4、CET-6、考研、托福、雅思、GRE"),
            Line::from("  • 学习流程：选择单词本 → 开始学习 → 单词进入词汇库"),
            Line::from(""),
            Line::from(vec![
                Span::styled("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", Theme::text_secondary()),
            ]),
            Line::from(vec![
                Span::styled("● SM2 算法（SuperMemo-2）", Theme::text_warning()),
            ]),
            Line::from(vec![
                Span::styled("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", Theme::text_secondary()),
            ]),
            Line::from(""),
            Line::from("  • 基于记忆曲线的智能复习算法"),
            Line::from("  • 根据您的回答质量动态调整复习间隔"),
            Line::from("  • 越熟悉的单词，复习间隔越长"),
            Line::from(""),
            Line::from(vec![
                Span::styled("  复习间隔示例：", Theme::text_secondary()),
            ]),
            Line::from("    第1次复习：1天后"),
            Line::from("    第2次复习：6天后"),
            Line::from("    第3次复习：根据质量动态计算（通常10-20天）"),
            Line::from("    ...持续延长间隔"),
            Line::from(""),
            Line::from(vec![
                Span::styled("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", Theme::text_secondary()),
            ]),
            Line::from(vec![
                Span::styled("● 评分等级（Review时按1-4评分）", Theme::text_warning()),
            ]),
            Line::from(vec![
                Span::styled("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", Theme::text_secondary()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(" 1 ", Theme::text_normal().bg(Theme::QUALITY_1)),
                Span::raw(" "),
                Span::styled("Hard", Theme::text_accent()),
                Span::raw(" - 完全不记得"),
            ]),
            Line::from("    → 重新开始学习，从1天后复习"),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(" 2 ", Theme::text_normal().bg(Theme::QUALITY_2)),
                Span::raw(" "),
                Span::styled("Difficult", Theme::text_warning()),
                Span::raw(" - 记得模糊"),
            ]),
            Line::from("    → 缩短复习间隔，增加练习频率"),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(" 3 ", Theme::text_normal().bg(Theme::QUALITY_3)),
                Span::raw(" "),
                Span::styled("Good", Theme::text_info()),
                Span::raw(" - 记得清楚"),
            ]),
            Line::from("    → 正常延长复习间隔"),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled(" 4 ", Theme::text_normal().bg(Theme::QUALITY_4)),
                Span::raw(" "),
                Span::styled("Easy", Theme::text_success()),
                Span::raw(" - 完全记得"),
            ]),
            Line::from("    → 大幅延长复习间隔"),
            Line::from(""),
            Line::from(vec![
                Span::styled("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", Theme::text_secondary()),
            ]),
            Line::from(vec![
                Span::styled("● 掌握标准", Theme::text_warning()),
            ]),
            Line::from(vec![
                Span::styled("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", Theme::text_secondary()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  • 当复习间隔达到 "),
                Span::styled("21天", Theme::text_title()),
                Span::raw(" 时，单词被标记为"),
                Span::styled("\"已掌握\"", Theme::text_success()),
            ]),
            Line::from("  • 已掌握的单词不会出现在待复习列表中"),
            Line::from("  • 如果评分选择1-2，将重新进入学习状态"),
            Line::from(""),
            Line::from(vec![
                Span::styled("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", Theme::text_secondary()),
            ]),
            Line::from(vec![
                Span::styled("💡 使用技巧", Theme::text_info()),
            ]),
            Line::from(vec![
                Span::styled("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━", Theme::text_secondary()),
            ]),
            Line::from(""),
            Line::from("  • 诚实评分很重要！评分越准确，复习效果越好"),
            Line::from("  • 完全不记得时选1，不要犹豫"),
            Line::from("  • 记得模糊时选2，而不是猜对后选3"),
            Line::from("  • 只有真正轻松回忆起来时才选4"),
            Line::from("  • 建议每天坚持完成目标，养成学习习惯"),
            Line::from(""),
        ];

        let content_height = help_lines.len() as u16;
        let help_block = Theme::block_with_title(" 📖 学习指南 (j/k 或 ↑/↓ 滚动) ");
        let help_inner = help_block.inner(chunks[2]);
        
        let help_widget = Paragraph::new(help_lines)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0))
            .block(help_block)
            .style(Theme::text_normal());
        frame.render_widget(help_widget, chunks[2]);

        // Scrollbar
        if content_height > help_inner.height {
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(Some("↑"))
                    .end_symbol(Some("↓")),
                help_inner.inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut ScrollbarState::new(content_height as usize)
                    .position(self.scroll as usize),
            );
        }
    }
}
