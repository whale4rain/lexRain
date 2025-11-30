# 开发文档

## 架构设计

### 组件系统
基于 Trait 的组件化设计，每个界面是独立的组件：

```rust
pub trait Component {
    fn handle_key(&mut self, key: KeyEvent) -> Result<Action>;
    fn view(&mut self, frame: &mut Frame, area: Rect);
}
```

### 数据库架构
**双数据库分离设计**：
- `dict_conn`: ECDICT 词典，只读，340万词条
- `learn_conn`: 学习进度，读写，learning_log + review_history

### 目录结构
```
src/
├── main.rs              # 入口 + 事件循环
├── app.rs/app_v2.rs     # 全局状态管理
├── db.rs                # 数据库操作
├── models.rs            # 数据结构（Word, LearningLog）
├── sm2.rs               # SM-2 算法
├── event.rs/tui.rs      # 事件处理 + TUI 初始化
└── components/          # 组件目录
    ├── dashboard.rs     # 主界面
    ├── review.rs        # 复习模式
    ├── wordbook.rs      # 单词本选择
    ├── dictionary.rs    # 词典
    ├── history.rs       # 历史
    ├── statistics.rs    # 统计
    └── common/          # 通用组件
        ├── progress_bar.rs
        ├── search_input.rs
        ├── status_bar.rs
        └── popup.rs     # 浮窗组件
```

## 核心技术

### 浮窗组件
```rust
// common/popup.rs
pub struct Popup {
    scroll: u16,
    title: String,
}

impl Popup {
    pub fn render(&mut self, frame: &mut Frame, area: Rect, content_lines: Vec<Line<'_>>) {
        // 居中显示（80% 宽度，90% 高度）
        let popup_area = centered_rect(80, 90, area);
        // 清除背景 + 渲染边框 + 内容 + 滚动条
    }
}

// 使用方式
if self.show_popup {
    let lines = self.build_detail_lines(word, log);
    self.popup.render(frame, area, lines);
}
```

### 单词本组件
```rust
// wordbook.rs
pub struct WordbookComponent {
    db: Database,
    wordbooks: Vec<(String, usize)>, // (tag, count)
    selected_index: usize,
    shuffle_mode: bool,
}

// 数据库方法
impl Database {
    pub fn get_wordbooks() -> Result<Vec<(String, usize)>> {
        // SELECT tag, COUNT(*) FROM stardict
        // WHERE tag IS NOT NULL AND tag != ''
        // GROUP BY tag ORDER BY count DESC
    }

    pub fn get_words_by_tag(tag: &str, limit: usize, shuffle: bool) -> Result<Vec<(Word, LearningLog)>> {
        // 乱序: ORDER BY RANDOM()
        // 顺序: ORDER BY oxford DESC, collins DESC, bnc ASC, frq ASC
        // WHERE tag LIKE '%' || tag || '%'
    }
}

// Review 模式
pub enum ReviewMode {
    Due,
    LearnNew,
    Wordbook(String, bool), // (tag, shuffle)
}
```

### Dictionary 搜索交互（v2.9）
**模式切换系统**：
```rust
#[derive(Debug, Clone, PartialEq)]
enum Mode {
    Normal,  // 导航模式：j/k/h/l 浏览
    Insert,  // 输入模式：输入搜索词
}

pub struct DictionaryComponent {
    mode: Mode,
    searching: bool,
    loading_frame: usize,
    // ...
}

// 键位处理
fn handle_normal_mode(&mut self, key: KeyEvent) -> Result<Action> {
    match key.code {
        KeyCode::Tab | KeyCode::Char('i') => self.mode = Mode::Insert,
        KeyCode::Char('j') => self.select_next(),
        KeyCode::Enter => self.show_popup = true,
        // ...
    }
}

fn handle_insert_mode(&mut self, key: KeyEvent) -> Result<Action> {
    match key.code {
        KeyCode::Tab | KeyCode::Esc => self.mode = Mode::Normal,
        KeyCode::Enter => {
            self.update_search()?;
            self.mode = Mode::Normal;  // 自动返回
        }
        KeyCode::Char(c) => self.search_input.handle_key(key),
        // ...
    }
}
```

**加载动画**：
```rust
// Braille 字符旋转动画
let loading_animation = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
let frame = loading_animation[self.loading_frame % loading_animation.len()];
format!(" Search [Enter to search] - {} Searching... ", frame)

// 每帧更新（view 方法中）
if self.searching {
    self.loading_frame = self.loading_frame.wrapping_add(1);
}
```

**动态提示**：
- Normal 模式：`Search [Tab to open]` + "Press Tab to open search..."
- Insert 模式：`Search [Enter to search]` + "Type and press Enter to search..."
- 搜索中：`Search [Enter to search] - ⠋ Searching...`

### SM-2 算法实现
```rust
// sm2.rs
pub fn process_review(log: &mut LearningLog, quality: u8) {
    if quality >= 3 {
        log.repetition += 1;
        log.interval = match log.repetition {
            1 => 1,
            2 => 6,
            _ => (log.interval as f32 * log.e_factor).round() as i64,
        };
    } else {
        log.repetition = 0;
        log.interval = 1;
    }
    log.e_factor = (log.e_factor + (0.1 - (5 - quality) as f32 * (0.08 + (5 - quality) as f32 * 0.02))).max(1.3);
    log.next_review = Utc::now() + Duration::days(log.interval);
}
```

### 智能选词算法
优先级排序（ORDER BY 子句）：
1. `oxford DESC` - 牛津3000核心词
2. `collins DESC` - 柯林斯星级
3. `bnc ASC` - BNC 词频（值越小频率越高）
4. `frq ASC` - 当代语料库词频

过滤条件：
- 必须有中文翻译（`translation IS NOT NULL`）
- 排除单字母（`length(spelling) > 1`，保留 a/I）
- 排除连字符词（`spelling NOT LIKE '%-%'`）
- 排除多词短语（`spelling NOT LIKE '% %'`）

### 双面板滚动系统
```rust
// ReviewComponent
pub struct ReviewComponent {
    scroll: u16,              // 释义面板滚动
    exchange_scroll: u16,      // 词形变化面板滚动
    active_panel: ActivePanel, // 当前焦点（Definition/Exchange）
}

// 滚动逻辑
match self.active_panel {
    ActivePanel::Definition => self.scroll = self.scroll.saturating_add(1),
    ActivePanel::Exchange => self.exchange_scroll = self.exchange_scroll.saturating_add(1),
}

// 面板切换
KeyCode::Char('h') | KeyCode::Left => {
    self.active_panel = ActivePanel::Definition;
}
KeyCode::Char('l') | KeyCode::Right | KeyCode::Tab => {
    self.active_panel = ActivePanel::Exchange;
}
```

## 数据模型

### Word 结构（ECDICT）
```rust
pub struct Word {
    pub id: Option<i64>,
    pub spelling: String,      // 单词拼写
    pub phonetic: Option<String>, // 音标
    pub definition: String,    // 英文定义
    pub translation: Option<String>, // 中文翻译
    pub pos: Option<String>,   // 词性（v:100/n:50）
    pub collins: i32,          // 柯林斯星级 (0-5)
    pub oxford: bool,          // 牛津3000标记
    pub tag: Option<String>,   // 考试标签（空格分隔）
    pub bnc: Option<i32>,      // BNC词频
    pub frq: Option<i32>,      // 当代语料库词频
    pub exchange: Option<String>, // 词形变化（p:went/d:gone）
}
```

### LearningLog 结构
```rust
pub struct LearningLog {
    pub word_id: i64,
    pub repetition: i32,       // 连续正确次数
    pub interval: i64,         // 间隔天数
    pub e_factor: f32,         // 难度系数 [1.3, 2.5]
    pub next_review: DateTime<Utc>,
    pub status: LearningStatus, // New/Learning/Mastered
}
```

## UI 布局方案

### Review 布局（Answer State）
```
┌─────────────────────────────────────────┐
│ Progress: 5/20 (15 remaining)           │ Length:3
├─────────────────────────────────────────┤
│ word             [ phonetic ]           │ Length:5
│ v. 动词  柯林斯★★  牛津3000             │
│ 考试: CET-4 · 考研                      │
├────────────────────┬────────────────────┤
│ 释义 [FOCUSED]     │ 词形变化           │ Min:10
│ ━━━ 中文释义 ━━━   │ 过去式             │
│   n. ...           │   went             │
│                    │ 过去分词           │
│ ━━━ English ━━━    │   gone             │
│   v. ...           │                    │
│                    │                    │
│ 词频: BNC:100      │                    │
│                [▲] │                    │
│                [█] │                    │
│                [▼] │                    │
└────────────────────┴────────────────────┘
  70%                  30%
```

### Dictionary 布局
```
┌─────────────────────────────────────────┐
│ [Search: percei_]                       │ Length:3
├─────────────────────────────────────────┤
│ ◯ │ perceive │ [pərˈsiːv] │ 5 days     │ Min:10
│ ◐ │ perception│ [pərˈsepʃn]│ 3 days    │
│ ● │ ...      │ ...        │ ...        │
│                                    [1/3]│
├─────────────────────────────────────────┤
│ Detail (h/l: scroll)                    │ Length:20
│ perceive  [ pərˈsiːv ]                  │
│ v. 动词  柯林斯★★★  牛津3000            │
│ 考试: CET-6 · 考研 · TOEFL              │
│ ━━━ 中文释义 ━━━                        │
│   vt. 感知；理解；察觉                  │
│ ━━━ English Definition ━━━              │
│   vt. To become aware of...             │
│ ━━━ 词形变化 ━━━                        │
│   过去式  perceived                     │
│   过去分词  perceived                   │
│   现在分词  perceiving                  │
│ 词频: BNC:500 | 当代:600                │
│                                      [█]│
└─────────────────────────────────────────┘
```

## 辅助函数

### 词性解析
```rust
fn parse_pos(pos: &str) -> String {
    // "v:100" → "v. 动词"
    // "n:50/v:30" → "n. 名词 / v. 动词"
    let parts: Vec<&str> = pos.split('/').collect();
    let mut result = Vec::new();
    for part in parts {
        if let Some((code, _weight)) = part.split_once(':') {
            let name = match code {
                "n" => "n. 名词", "v" => "v. 动词",
                "j"|"a"|"adj" => "adj. 形容词",
                "r"|"ad"|"adv" => "adv. 副词",
                // ... 11 种词性
            };
            result.push(name);
        }
    }
    result.join(" / ")
}
```

### Exchange 解析
```rust
fn parse_exchange(exchange: &str) -> HashMap<&str, String> {
    // "p:went/d:gone/i:going" → {p: "went", d: "gone", i: "going"}
    let mut map = HashMap::new();
    for part in exchange.split('/') {
        if let Some((key, value)) = part.split_once(':') {
            map.insert(key, value.to_string());
        }
    }
    map
}

fn exchange_type_name(key: &str) -> &str {
    match key {
        "p" => "过去式", "d" => "过去分词", "i" => "现在分词",
        "3" => "第三人称单数", "s" => "复数",
        "r" => "比较级", "t" => "最高级",
        "0" => "原型", "1" => "原型变换",
        _ => key,
    }
}
```

## 键位设计原则

1. **一致性**: j/k 在所有组件表示向下/向上
2. **语义化**: h/l 根据上下文有合理的不同含义
3. **可发现性**: 标题栏显示操作提示
4. **Vim 兼容**: hjkl + g/G + 数字键
5. **模态切换**: Enter 打开浮窗，q/Esc 关闭

### 浮窗模式
- Dictionary/History 按 Enter 进入浮窗模式
- 浮窗模式下 j/k 滚动，q/Esc 退出
- 浮窗居中显示（80%×90%），自动滚动条

## 性能优化

### 数据库查询
- ECDICT 只读，无写操作锁竞争
- **索引优化**（v2.9）：
  - `CREATE INDEX idx_word ON stardict(word)` - 单词精确查询
  - `CREATE INDEX idx_tag ON stardict(tag)` - 标签筛选
  - 查询速度提升 10-100 倍（大数据集）
- 分页查询（30 词/页）减少数据传输
- LIMIT 限制查询结果（默认 20/100）

### 事件处理优化
- **轮询优化**（v2.9）：event poll 从 50ms → 10ms
  - 按键响应更快（特别是 q 退出）
  - Tick rate 降低不影响性能（event-driven）
- KeyEventKind::Press 过滤，避免重复处理

### 渲染优化
- 滚动条仅在内容超长时渲染
- 使用 `saturating_add/sub` 避免溢出检查
- 组件状态按需更新
- **加载动画**（v2.9）：Braille 字符旋转，无额外性能开销

### 内存使用
- Word 结构按需加载，不常驻内存
- 复习队列 Vec 动态增长
- 滚动位置用 u16（2字节）

## 测试建议

### 单元测试
- SM-2 算法各评分逻辑
- Exchange/POS 解析函数
- 智能选词 SQL 查询

### 集成测试
- 双数据库连接
- Review 完整流程（问题→答案→评分→下一个）
- 滚动边界条件

### 手动测试
- 长定义滚动（>30行）
- 无词形变化的单词
- 搜索结果为空
- 首次启动（无进度数据）

## 新功能开发流程

1. **设计阶段**
   - 在 FEATURES.md 添加功能描述（简短，3-5 条）
   - 确定键位绑定（避免冲突）
   - 设计数据结构（如需新表）

2. **实现阶段**
   - 在对应组件文件添加代码
   - 更新 handle_key() 和 view()
   - 添加必要的辅助函数

3. **测试阶段**
   - cargo check 编译检查
   - 手动测试核心路径
   - 验证键位无冲突

4. **文档阶段**
   - 更新 FEATURES.md 版本记录（简要，不超过 5 条）
   - 在本文件（DEVELOPMENT.md）添加技术细节（仅复杂实现）
   - 更新 README.md 键位表（如有新键位）
   - **不要创建新的 MD 文件**

## 文档维护原则

### 禁止行为
- ❌ 为每个功能创建单独的 MD 文件
- ❌ 写超过 1 页的详细描述
- ❌ 重复已有文档的内容
- ❌ 添加不必要的截图和图表

### 推荐行为
- ✅ 使用列表和表格代替段落
- ✅ 代码示例控制在 10-20 行
- ✅ 技术细节归类到本文件的对应章节
- ✅ 版本记录只记录关键变更点

## 依赖库

```toml
[dependencies]
ratatui = "0.25"          # TUI 框架
crossterm = "0.27"        # 终端控制
rusqlite = { version = "0.30", features = ["bundled"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"            # 错误处理
chrono = "0.4"            # 时间处理
```

## 常见问题

**Q: 如何添加新的键位绑定？**  
A: 在对应组件的 `handle_key()` 方法添加 `KeyCode::Char('x')` 匹配

**Q: 如何修改布局比例？**  
A: 修改 `Layout::default().constraints([Constraint::Percentage(70), Constraint::Percentage(30)])`

**Q: 如何添加新的元数据字段？**  
A: 1) 更新 Word 结构 2) 更新 SQL 查询 3) 更新 view() 渲染逻辑

**Q: 如何添加新的统计图表？**  
A: 在 statistics.rs 的 view() 添加新的 Chart 组件

## 相关资源

- Ratatui 文档: https://ratatui.rs/
- ECDICT 项目: https://github.com/skywind3000/ECDICT
- SM-2 算法: https://www.supermemo.com/en/archives1990-2015/english/ol/sm2
