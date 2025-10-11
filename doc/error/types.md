# error/types.rs - 错误类型定义

## 模块概述

定义了下载管理器的所有错误类型，使用 `thiserror` crate 提供详细的错误信息。

## 错误枚举

### DownloadError
- **位置**: src/error/types.rs:6
- **说明**: 下载管理器的主要错误枚举，涵盖所有可能的错误情况

#### 错误变体

##### TaskNotFound(TaskId)
- **位置**: src/error/types.rs:8
- **说明**: 找不到指定ID的任务
- **参数**: `TaskId` - 未找到的任务ID

##### InvalidStatusTransition
- **位置**: src/error/types.rs:11
- **说明**: 无效的任务状态转换
- **用途**: 当试图进行不允许的状态变更时抛出

##### ConcurrencyLimitExceeded
- **位置**: src/error/types.rs:14
- **说明**: 超过最大并发下载数限制
- **用途**: 保护系统资源，避免过多并发下载

##### InvalidUrl(String)
- **位置**: src/error/types.rs:17
- **说明**: 无效的URL格式
- **参数**: `String` - 导致错误的URL

##### InvalidPath(String)
- **位置**: src/error/types.rs:20
- **说明**: 无效的目标路径
- **参数**: `String` - 导致错误的路径

##### DownloaderUnavailable(String)
- **位置**: src/error/types.rs:23
- **说明**: 下载器不可用
- **参数**: `String` - 错误详情

##### IoError(std::io::Error)
- **位置**: src/error/types.rs:26
- **说明**: IO操作错误
- **用途**: 自动从标准库的IO错误转换而来

##### DatabaseError(String)
- **位置**: src/error/types.rs:29
- **说明**: 数据库操作错误
- **参数**: `String` - 数据库错误详情

##### General(String)
- **位置**: src/error/types.rs:32
- **说明**: 通用错误类型
- **参数**: `String` - 错误描述

##### DuplicateDetectionError(String)
- **位置**: src/error/types.rs:36
- **说明**: 重复检测失败
- **参数**: `String` - 检测失败的详细信息
- **用途**: 重复下载检测功能相关错误

##### VerificationError(String)
- **位置**: src/error/types.rs:39
- **说明**: 任务验证失败
- **参数**: `String` - 验证失败的详细信息

##### PolicyViolation { task_id: TaskId, reason: String }
- **位置**: src/error/types.rs:42
- **说明**: 策略违规错误
- **参数**:
  - `task_id: TaskId` - 重复任务的ID
  - `reason: String` - 违规原因
- **用途**: 当发现重复任务且违反设定策略时抛出

## 依赖项

- `thiserror::Error` - 用于自动生成错误特征实现
- `crate::types::TaskId` - 任务ID类型