# models/duplicate_policy.rs - 重复处理策略

## 枚举类型

### DuplicatePolicy
- **位置**: src/models/duplicate_policy.rs:9
- **说明**: 定义当检测到重复下载时系统应该如何处理的策略

#### 策略变体

##### ReuseExisting
- **位置**: src/models/duplicate_policy.rs:11
- **说明**: 重用现有任务，无论状态如何（默认策略）
- **用途**: 最宽松的策略，总是重用已存在的任务

##### AllowDuplicate
- **位置**: src/models/duplicate_policy.rs:12
- **说明**: 总是创建新任务，忽略重复
- **用途**: 允许完全重复的下载

##### PromptUser
- **位置**: src/models/duplicate_policy.rs:14
- **说明**: 发现重复时询问用户决定
- **用途**: 交互式处理，让用户选择如何处理重复

##### ReuseIfComplete
- **位置**: src/models/duplicate_policy.rs:16
- **说明**: 仅当原任务已完成时重用
- **用途**: 避免干扰未完成的下载

##### ReuseIfIncomplete
- **位置**: src/models/duplicate_policy.rs:18
- **说明**: 仅当原任务未完成时重用（用于恢复）
- **用途**: 恢复中断的下载

##### FailIfDuplicate
- **位置**: src/models/duplicate_policy.rs:20
- **说明**: 发现重复时返回错误
- **用途**: 严格防止重复下载

## 方法实现

### allows_reuse(status)
- **位置**: src/models/duplicate_policy.rs:32
- **功能**: 检查此策略是否允许重用给定状态的任务
- **参数**: `status: &crate::models::TaskStatus` - 任务状态
- **返回值**: `bool` - 是否允许重用
- **说明**: 根据策略和任务状态判断是否可以重用现有任务

### should_fail_on_duplicate()
- **位置**: src/models/duplicate_policy.rs:53
- **功能**: 检查此策略是否应在发现重复时失败
- **返回值**: `bool` - 是否应该失败
- **说明**: 仅FailIfDuplicate策略返回true

### requires_user_decision()
- **位置**: src/models/duplicate_policy.rs:58
- **功能**: 检查此策略是否需要用户交互
- **返回值**: `bool` - 是否需要用户决定
- **说明**: 仅PromptUser策略返回true

## 特征实现

### Default
- **位置**: src/models/duplicate_policy.rs:24
- **说明**: 默认策略为ReuseExisting

### Debug, Clone, PartialEq, Eq, Serialize, Deserialize
- **位置**: src/models/duplicate_policy.rs:8
- **说明**: 提供调试、克隆、比较和序列化支持

## 策略决策逻辑

该策略枚举提供了灵活的重复检测处理机制：

1. **ReuseExisting**: 适用于大多数场景，避免重复工作
2. **AllowDuplicate**: 适用于需要多次下载同一文件的场景
3. **PromptUser**: 适用于交互式应用，让用户控制行为
4. **ReuseIfComplete**: 避免影响正在进行的下载
5. **ReuseIfIncomplete**: 专门用于恢复中断的下载
6. **FailIfDuplicate**: 用于严格控制，防止任何重复

## 依赖项

- `serde::{Deserialize, Serialize}` - 序列化支持
- `crate::models::TaskStatus` - 任务状态类型