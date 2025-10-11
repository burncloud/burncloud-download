# BurnCloud Download 项目文档

本目录包含 BurnCloud Download 项目的完整代码文档，按照 `src` 目录结构组织。

## 目录结构

```
doc/
├── lib.md                          # 核心库文档 (lib.rs)
├── error/                          # 错误处理模块
│   ├── mod.md                      # 错误模块入口
│   └── types.md                    # 错误类型定义
├── manager/                        # 下载管理器模块
│   ├── mod.md                      # 管理器模块入口
│   ├── basic.md                    # 基础下载管理器
│   └── persistent_aria2.md         # 持久化Aria2管理器
├── models/                         # 数据模型模块
│   ├── mod.md                      # 模型模块入口
│   ├── duplicate_policy.md         # 重复处理策略
│   ├── duplicate_reason.md         # 重复检测原因
│   ├── duplicate_result.md         # 重复检测结果
│   ├── file_identifier.md          # 文件标识符
│   └── task_status.md              # 任务状态
├── queue/                          # 队列管理模块
│   └── mod.md                      # 队列模块入口
├── services/                       # 服务模块
│   └── mod.md                      # 服务模块入口
├── traits/                         # 特征定义模块
│   └── mod.md                      # 特征模块入口
├── types/                          # 类型定义模块
│   └── mod.md                      # 类型模块入口
└── utils/                          # 工具模块
    └── mod.md                      # 工具模块入口
```

## 核心模块说明

### [lib.md](lib.md) - 核心库
- 全局便利函数（download, download_to等）
- 全局管理器实例
- 类型重新导出

### [manager/](manager/) - 下载管理器
- **basic.md**: 基础下载管理器，用于演示和测试
- **persistent_aria2.md**: 生产环境使用的持久化Aria2管理器

### [models/](models/) - 数据模型
- **duplicate_policy.md**: 重复下载处理策略
- **duplicate_reason.md**: 重复检测原因分类
- **duplicate_result.md**: 重复检测结果类型
- **file_identifier.md**: 文件标识符，用于重复检测
- **task_status.md**: 扩展的任务状态

### [error/](error/) - 错误处理
- **types.md**: 完整的错误类型定义

### 其他模块
- **[traits/](traits/)**: 核心特征定义
- **[services/](services/)**: 重复检测等服务
- **[queue/](queue/)**: 任务队列管理
- **[types/](types/)**: 基础类型（主要重新导出）
- **[utils/](utils/)**: 工具函数

## 关键概念

### 下载管理器
- `BasicDownloadManager`: 简单的内存管理器，用于测试
- `PersistentAria2Manager`: 生产级管理器，支持持久化和恢复

### 重复检测
- 基于URL哈希和目标路径的智能重复检测
- 支持多种重复处理策略
- 详细的重复原因分类

### 错误处理
- 统一的错误类型系统
- 详细的错误信息和分类
- 支持错误链和上下文

## 使用指南

1. **简单使用**: 查看 [lib.md](lib.md) 了解便利函数
2. **高级使用**: 查看 [manager/persistent_aria2.md](manager/persistent_aria2.md) 了解完整功能
3. **错误处理**: 查看 [error/types.md](error/types.md) 了解错误类型
4. **重复检测**: 查看 [models/](models/) 目录了解重复检测机制

## 文档生成时间

此文档基于源码 `src/` 目录结构自动生成，每个函数都包含详细的功能说明、参数描述和使用示例。