# LexRain - 基于 SuperMemo-2 算法的单词记忆应用

基于 Rust 和 TUI 的单词记忆应用，采用 SM-2 间隔重复算法 + ECDICT 340万词库。

## 特性

- **SM-2 算法**: 科学的间隔重复记忆法
- **ECDICT 词库**: 340万+ 词条，柯林斯星级，牛津3000，考试标签
- **双面板滚动**: 释义和词形变化独立滚动
- **智能选词**: 优先牛津3000、柯林斯高星、BNC高频词
- **本地存储**: SQLite 双数据库架构，隐私保护

## 快速开始

```bash
# 1. 下载 ECDICT 数据库 (https://github.com/skywind3000/ECDICT/releases)
# 2. 解压到 ecdict-sqlite-28/stardict.db
# 3. 运行
cargo run --release

# 操作: Dashboard 按 n (新词) | r (复习) | d (词典) | h (历史) | q (退出)
# Review: Space (显示答案) | 1-4 (评分) | j/k (滚动) | h/l (切换面板)
```

## 架构

### 双数据库
- `ecdict-sqlite-28/stardict.db` - ECDICT 词典（只读，340万词条）
- `lexrain_progress.db` - 学习进度（读写）

### 组件系统
- `Dashboard` - 主界面，统计信息
- `Review` - 复习模式，双面板滚动（释义 70% + 词形变化 30%）
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
| `g/G` | 首/尾 |
| 字母 | 搜索 |
| `q` | 返回 |

### History 历史
| 按键 | 功能 |
|------|------|
| `j/k` | 上下移动 |
| `PageUp/Down` | 翻页 |
| `g/G` | 首/尾 |
| `q` | 返回 |

## 技术栈

Rust + ratatui + crossterm + rusqlite + ECDICT

## 版本历史

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
