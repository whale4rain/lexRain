# LexRain - 基于 SuperMemo-2 算法的单词记忆应用

一个基于 Rust 和 TUI (Terminal User Interface) 的单词记忆应用，采用 SuperMemo-2 (SM-2) 间隔重复算法，帮助你高效记忆单词。

## 特性

- **科学记忆算法**: 采用经典的 SuperMemo-2 算法，根据遗忘曲线优化复习间隔
- **本地存储**: 使用 SQLite 数据库，所有数据本地存储，保护隐私
- **简洁 TUI**: 基于 ratatui 的终端界面，清爽高效
- **灵活导入**: 支持从 JSON 文件批量导入词库
- **实时统计**: 显示学习进度和掌握程度

## 目录结构

```
src/
├── main.rs           # 入口与事件循环
├── app.rs            # 全局状态管理
├── ui.rs             # 界面渲染逻辑
├── db.rs             # SQLite 操作封装
├── models.rs         # 数据结构定义
├── sm2.rs            # SuperMemo-2 算法实现
├── event.rs          # 事件处理系统
└── tui.rs            # TUI 初始化
```

## 数据库结构

### words 表 (词库表)
| Field | Type | Description |
| :--- | :--- | :--- |
| id | INTEGER PK | 自增 ID |
| spelling | TEXT | 单词拼写 (Unique) |
| phonetic | TEXT | 音标 |
| definition | TEXT | 释义 |
| tags | TEXT | 标签 (如 "CET-4", "TOEFL") |

### learning_log 表 (学习记录表)
| Field | Type | Description |
| :--- | :--- | :--- |
| word_id | INTEGER FK | 外键关联 words |
| repetition | INTEGER | 当前连续正确次数 |
| interval | INTEGER | 当前间隔天数 |
| e_factor | REAL | 当前难度系数 |
| next_review | DATETIME | 下次复习时间戳 |
| status | INTEGER | 学习状态 (0:新词, 1:学习中, 2:已掌握) |

## SuperMemo-2 算法说明

### 核心参数

- **Repetition (n)**: 连续正确次数
- **Interval (I)**: 距离下次复习的天数
- **E-Factor (EF)**: 难度系数（初始 2.5）

### 评分系统 (1-4)

- **1**: Forgot (忘记) - 重置进度，间隔归 1
- **2**: Hard (困难) - 记住但费力
- **3**: Good (良好) - 正常记住
- **4**: Easy (简单) - 轻松记住

### 算法逻辑

质量评分 >= 3 时：
- 第一次复习：间隔 1 天
- 第二次复习：间隔 6 天
- 之后：间隔 = 上次间隔 × E-Factor

质量评分 < 3 时：
- 重置 repetition = 0
- 间隔归 1 天

E-Factor 更新公式：
```
EF' = EF + (0.1 - (5 - quality) * (0.08 + (5 - quality) * 0.02))
EF 最小值为 1.3
```

## 安装与使用

### 1. 编译项目

```bash
cargo build --release
```

### 2. 导入词库

项目自带 `sample_words.json` 示例词库，包含 20 个常用词汇：

```bash
cargo run --release -- --import sample_words.json
```

### 3. 启动应用

```bash
cargo run --release
```

## 词库格式

JSON 数组格式，每个单词包含以下字段：

```json
[
  {
    "spelling": "algorithm",
    "phonetic": "ˈælɡərɪðəm",
    "definition": "n. A step-by-step procedure for solving a problem...",
    "tags": "programming,computer-science"
  }
]
```

字段说明：
- `spelling`: 必填，单词拼写
- `phonetic`: 可选，音标
- `definition`: 必填，释义
- `tags`: 可选，标签（逗号分隔）

## 界面操作

### Dashboard (主界面)
- `r` - 开始复习
- `d` - 词典（开发中）
- `q` - 退出应用

### Review (复习模式)
- **问题阶段**:
  - `Space` / `Enter` - 显示答案
  - `Esc` / `q` - 返回主界面

- **答案阶段**:
  - `1` - Forgot (忘记)
  - `2` - Hard (困难)
  - `3` - Good (良好)
  - `4` - Easy (简单)

## 推荐词库资源

以下是一些高质量的开源词库：

1. **[ECDICT](https://github.com/skywind3000/ECDICT)**
   最全的英汉词典数据库，包含数百万词条

2. **[墨墨背单词导出](https://github.com/linonetwo/maimemo-export)**
   包含雅思、考研、四六级等特定考试词表

3. **[k_english_vocabulary](https://github.com/kajweb/english_vocabulary)**
   结构化较好的常用单词 JSON

## 技术栈

- **Rust 2021 Edition**
- **ratatui** - TUI 框架
- **crossterm** - 跨平台终端控制
- **rusqlite** - SQLite 数据库
- **serde** - 序列化/反序列化
- **chrono** - 时间处理
- **clap** - 命令行参数解析

## 开发计划

- [x] SM-2 算法实现
- [x] SQLite 本地存储
- [x] TUI 界面
- [x] 词库导入
- [x] 复习模式
- [ ] 词典搜索功能
- [ ] 更多词库源支持
- [ ] 学习曲线可视化
- [ ] 导出学习报告

## License

MIT

## 贡献

欢迎提交 Issue 和 Pull Request！
