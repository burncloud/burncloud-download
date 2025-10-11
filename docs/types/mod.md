# types/mod.rs - 类型模块

## 模块概述

类型模块的入口文件。注意：主要类型定义已移至 burncloud-download-types crate。

## 重新导出

该模块主要重新导出来自 burncloud-download-types 的核心类型：
- TaskId - 任务唯一标识符
- DownloadTask - 下载任务结构
- DownloadProgress - 下载进度信息
- DownloadStatus - 下载状态枚举