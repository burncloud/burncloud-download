# manager/mod.rs - 管理器模块入口

## 模块概述

管理器模块包含不同的下载管理器实现，提供了基础和持久化两种实现方式。

## 子模块

### basic
- **文件**: basic.rs
- **说明**: 基础下载管理器实现，用于演示和测试

### persistent_aria2
- **文件**: persistent_aria2.rs
- **说明**: 持久化的Aria2下载管理器，支持数据库持久化

## 重新导出

### BasicDownloadManager
- **来源**: basic::BasicDownloadManager
- **说明**: 基础下载管理器的公共接口

### PersistentAria2Manager
- **来源**: persistent_aria2::PersistentAria2Manager
- **说明**: 持久化Aria2管理器的公共接口