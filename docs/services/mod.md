# services/mod.rs - 重复检测服务模块

## 模块概述

该模块包含实现重复检测逻辑并与下载管理器协调的核心服务。

## 子模块

### duplicate_detector
- **文件**: duplicate_detector.rs
- **说明**: 重复检测器，核心重复检测逻辑

### task_repository
- **文件**: task_repository.rs
- **说明**: 任务仓库，数据库操作服务

### hash_calculator
- **文件**: hash_calculator.rs
- **说明**: 后台哈希计算器

### task_validation
- **文件**: task_validation.rs
- **说明**: 任务验证服务

## 重新导出

### DuplicateDetector
- **来源**: duplicate_detector::DuplicateDetector
- **说明**: 重复检测器服务

### TaskRepository
- **来源**: task_repository::TaskRepository
- **说明**: 任务仓库服务

### BackgroundHashCalculator
- **来源**: hash_calculator::BackgroundHashCalculator
- **说明**: 后台哈希计算服务

### TaskValidation
- **来源**: task_validation::TaskValidation
- **说明**: 任务验证服务