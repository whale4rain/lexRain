# LexRain 快速开始指南

## 📦 准备工作

### 1. 下载 ECDICT 数据库

访问：https://github.com/skywind3000/ECDICT/releases/tag/1.0.28

下载：`ecdict-sqlite-28.zip` (~700MB)

### 2. 解压数据库

解压到项目根目录，确保路径为：`ecdict-sqlite-28/stardict.db`

### 3. 运行程序

```bash
# Windows
.\target\release\lexRain.exe

# Linux/macOS
./target/release/lexRain
```

## 🎮 基本操作

### 首次使用

1. 程序启动 → Dashboard
2. 按 `n` → 学习新单词（自动选取20个高质量词汇）
3. 按 `Space` → 显示中英文释义
4. 按 `1-4` → 评价记忆质量

### 快捷键

**Dashboard**: `r` 复习 | `n` 新词 | `d` 词典 | `h` 历史 | `s` 统计 | `q` 退出

**Review**: `Space` 显示答案 | `1-4` 评分 | `q` 返回

**Dictionary**: 输入搜索 | `↑/↓` 或 `j/k` 移动 | `g/G` 首尾 | `q` 返回

## 💡 评分标准

- **4 (Easy)**: 瞬间回忆，非常熟悉
- **3 (Good)**: 稍有思考，能回忆起
- **2 (Hard)**: 勉强想起，不流利
- **1 (Forgot)**: 完全忘记

## 📚 ECDICT 特性

- **340万+ 词条**，双语释义
- **柯林斯星级**，牛津3000标注
- **考试标签**：四六级/托福/雅思/GRE
- **词频数据**：智能选词

## 🔧 常见问题

**Q**: 找不到数据库？  
**A**: 确保 `ecdict-sqlite-28/stardict.db` 路径正确

**Q**: 如何备份？  
**A**: 复制 `lexrain_progress.db` 文件

## 🎯 学习目标

- 📕 100词 → 📗 500词 → 📘 1000词 → 📙 3000词

---

更多详情：[README.md](./README.md) | [ECDICT_INTEGRATION.md](./ECDICT_INTEGRATION.md)
