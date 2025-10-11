# models/mod.rs - 重复检测数据模型模块

## 模块概述

该模块包含重复检测系统使用的数据结构，用于识别和管理 burncloud-download 系统中的重复下载。

## 子模块

### file_identifier
- **文件**: file_identifier.rs
- **说明**: 文件标识符，用于重复检测的组合键

### task_status
- **文件**: task_status.rs
- **说明**: 扩展的任务状态，支持重复检测

### duplicate_policy
- **文件**: duplicate_policy.rs
- **说明**: 重复处理策略

### duplicate_result
- **文件**: duplicate_result.rs
- **说明**: 重复检测操作的结果类型

### duplicate_reason
- **文件**: duplicate_reason.rs
- **说明**: 重复检测的原因

## 重新导出

### FileIdentifier
- **来源**: file_identifier::FileIdentifier
- **说明**: 用于重复检测的文件标识符

### TaskStatus
- **来源**: task_status::TaskStatus
- **说明**: 扩展的任务状态枚举

### DuplicatePolicy
- **来源**: duplicate_policy::DuplicatePolicy
- **说明**: 重复下载处理策略

### DuplicateResult 和 DuplicateAction
- **来源**: duplicate_result::{DuplicateResult, DuplicateAction}
- **说明**: 重复检测结果和建议操作

### DuplicateReason
- **来源**: duplicate_reason::DuplicateReason
- **说明**: 重复检测的原因枚举