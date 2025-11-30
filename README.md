# LexRain - 基于 SuperMemo-2 算法的单词记忆应用

基于 Rust 和 TUI 的单词记忆应用，采用 SM-2 间隔重复算法 + ECDICT 340万词库。

## 特性

- **SM-2 算法**: 科学的间隔重复记忆法
- **ECDICT 词库**: 340万+ 词条，柯林斯星级，牛津3000，考试标签
- **单词本分类**: 按考试标签（CET-4/6、TOEFL、IELTS、GRE 等）分类复习
- **双面板滚动**: 释义和词形变化独立滚动
- **智能选词**: 优先牛津3000、柯林斯高星、BNC高频词
- **本地存储**: SQLite 双数据库架构，隐私保护

## 快速开始

```bash
# 1. 下载 ECDICT 数据库 (https://github.com/skywind3000/ECDICT/releases)
# 2. 解压到 ecdict-sqlite-28/stardict.db
# 3. 运行
cargo run --release

# 操作: Dashboard 按 r (复习) | w (单词本) | d (词典) | h (历史) | q (退出)
# Review: Space (显示答案) | 1-4 (评分) | j/k (滚动) | h/l (切换面板)
```

## 架构

### 双数据库
- `ecdict-sqlite-28/stardict.db` - ECDICT 词典（只读，340万词条）
- `lexrain_progress.db` - 学习进度（读写）

### 组件系统
- `Dashboard` - 主界面，统计信息
- `Review` - 复习模式，双面板滚动（释义 70% + 词形变化 30%）
- `Wordbook` - 单词本选择，按标签分类（支持乱序）
- `Dictionary` - 词典搜索，完整元数据显示
- `History` - 复习历史记录
- `Statistics` - 学习统计图表

## SM-2 算法

- **评分**: 1 (忘记) | 2 (困难) | 3 (良好) | 4 (简单)
- **间隔**: 1天 → 6天 → I × EF (质量>=3), 否则重置为1天
- **EF**: 难度系数，范围 [1.3, 2.5]，根据评分动态调整

## 键位绑定

### Review 复习
| 按键 | 功能 |
|------|------|
| `Space` | 显示答案 |
| `j/k` | 滚动当前面板 |
| `h/l/Tab` | 切换面板焦点（释义 ↔ 词形变化）|
| `1-4` | 评分（1:忘记 2:困难 3:良好 4:简单）|
| `q` | 返回 |

### Dictionary 词典
| 按键 | 功能 |
|------|------|
| `j/k` | 上下选词 |
| `h/l` | 滚动详情 |
| `Enter` | 打开浮窗查看完整信息 |
| `g/G` | 首/尾 |
| 字母 | 搜索 |
| `q` | 返回 |

### History 历史
| 按键 | 功能 |
|------|------|
| `j/k` | 上下移动 |
| `Enter` | 打开浮窗查看详情 |
| `PageUp/Down` | 翻页 |
| `g/G` | 首/尾 |
| `q` | 返回 |

### 浮窗操作（Dictionary/History）
| 按键 | 功能 |
|------|------|
| `j/k` | 上下滚动 |
| `q` | 关闭浮窗 |

### Wordbook 单词本
| 按键 | 功能 |
|------|------|
| `↑/↓` `j/k` | 选择单词本 |
| `s` | 切换乱序/顺序模式 |
| `Enter` | 开始复习选中单词本 |
| `g/G` | 跳转到首/尾 |
| `PageUp/Down` | 翻页 |
| `q` | 返回 |

## 技术栈

Rust + ratatui + crossterm + rusqlite + ECDICT

## 版本历史

- **v2.6** (2025-12-01): 单词本显示增强
  - Review 界面显示当前单词本（带首字母图标）
  - Dashboard 添加单词本统计卡片
  - 移除 Learn New 功能，统一使用单词本复习
- **v2.5** (2025-12-01): 单词本功能（按标签分类，支持乱序）
- **v2.4** (2025-11-30): 浮窗功能（Dictionary/History 按 Enter）
- **v2.3** (2025-11-30): 双面板独立滚动，Dictionary 完整元数据
- **v2.2** (2025-11-30): ECDICT 元数据显示，词性解析
- **v2.1** (2025-11-30): Review/History 滚动功能
- **v2.0** (2025-11-30): ECDICT 集成，双数据库架构
- **v1.0** (2025-11): 初始版本，SM-2 算法

## 相关文档

- **[QUICKSTART.md](./QUICKSTART.md)** - 5分钟快速上手
- **[FEATURES.md](./FEATURES.md)** - 功能清单和版本历史
- **[DEVELOPMENT.md](./DEVELOPMENT.md)** - 架构设计和开发指南
- **[CLAUDE.md](./CLAUDE.md)** - AI 协作说明

## License

MIT
