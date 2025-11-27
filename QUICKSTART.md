# 快速开始指南

## 1. 第一次使用

### 编译项目
```bash
cargo build --release
```

### 导入示例词库
```bash
cargo run --release -- --import sample_words.json
```

成功后会显示：
```
Importing from sample_words.json...
Successfully imported 20 words!
```

## 2. 启动应用

```bash
cargo run --release
```

## 3. 界面说明

启动后你会看到主界面（Dashboard），显示：

```
┌────────────────── LexRain ──────────────────┐
│ Dashboard | Review | Dictionary | Quit      │
└──────────────────────────────────────────────┘

┌─────────────── Statistics ──────────────────┐
│ Total Words: 20 | Mastered: 0 | Due Today: 20│
└──────────────────────────────────────────────┘

┌────────────── Mastery Progress ─────────────┐
│ ████████░░░░░░░░░░░░░░░░░░░░░░░░░  0%       │
└──────────────────────────────────────────────┘

┌──────────────── Actions ────────────────────┐
│ Press 'r' to start Review                    │
│ Press 'd' for Dictionary                     │
│ Press 'q' to Quit                            │
└──────────────────────────────────────────────┘
```

## 4. 开始复习

### 按 'r' 进入复习模式

你会看到类似这样的界面：

```
┌─────────────────── Review ──────────────────┐
│                                              │
│              algorithm                       │
│                                              │
│            [ ˈælɡərɪðəm ]                   │
│                                              │
│        Press <Space> to show definition      │
│                                              │
└──────────────────────────────────────────────┘

Space: Show Answer | q: Quit
```

### 查看释义

按空格键后会显示释义：

```
┌─────────────────── Review ──────────────────┐
│                                              │
│              algorithm                       │
│                                              │
│            [ ˈælɡərɪðəm ]                   │
│                                              │
│ ─────────────── Definition ──────────────── │
│ n. A step-by-step procedure for solving a   │
│ problem or accomplishing a task, especially  │
│ in computer science and mathematics.         │
└──────────────────────────────────────────────┘

1: Forgot | 2: Hard | 3: Good | 4: Easy
```

### 评分

根据你的记忆情况选择：
- **1** - 完全不记得这个单词
- **2** - 记得但想了很久
- **3** - 比较快想起来了
- **4** - 立刻就知道意思

评分后会自动显示下一个单词。

## 5. 记忆原理

### SuperMemo-2 算法

应用会根据你的评分自动调整复习间隔：

- **第一次评 3/4**：1 天后再复习
- **第二次评 3/4**：6 天后再复习
- **之后每次评 3/4**：间隔逐渐拉长（基于难度系数）
- **评 1/2**：重新开始，明天复习

### 学习状态

- **New (新词)**：刚导入还没复习过
- **Learning (学习中)**：已经复习过但还没完全掌握
- **Mastered (已掌握)**：间隔超过 21 天，认为已经掌握

## 6. 制作自己的词库

创建一个 JSON 文件，例如 `my_words.json`：

```json
[
  {
    "spelling": "serendipity",
    "phonetic": "ˌserənˈdɪpɪti",
    "definition": "n. The occurrence and development of events by chance in a happy or beneficial way.",
    "tags": "vocabulary,advanced"
  },
  {
    "spelling": "ephemeral",
    "phonetic": "ɪˈfemərəl",
    "definition": "adj. Lasting for a very short time.",
    "tags": "vocabulary,literature"
  }
]
```

然后导入：

```bash
cargo run --release -- --import my_words.json
```

## 7. 常见问题

### Q: 数据存储在哪里？
A: 在项目根目录的 `lexrain.db` SQLite 数据库文件中。

### Q: 如何重新开始？
A: 删除 `lexrain.db` 文件，然后重新导入词库。

### Q: 可以多次导入吗？
A: 可以。相同拼写的单词会被忽略（INSERT OR IGNORE）。

### Q: 如何查看今天有多少单词要复习？
A: Dashboard 界面的 "Due Today" 会显示当天到期的单词数量。

### Q: 为什么刚导入的单词都显示 "Due Today"？
A: 新导入的单词初始的 `next_review` 时间是导入时刻，所以会立即出现在复习列表中。

## 8. 学习建议

1. **每天坚持**：即使只有 5 分钟，每天复习效果最好
2. **诚实评分**：不要高估自己，诚实评分才能让算法发挥最佳效果
3. **适量导入**：建议从 20-50 个单词开始，逐步增加
4. **及时复习**：看到 "Due Today" 有单词时尽快复习

## 9. 下一步

- 探索更多词库资源（参考 README.md）
- 制作个性化词库
- 每天坚持复习，见证进步！

祝你学习愉快！
