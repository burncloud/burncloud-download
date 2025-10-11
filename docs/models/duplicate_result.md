# models/duplicate_result.rs - 重复检测结果类型

## 枚举类型

### DuplicateResult
- **位置**: src/models/duplicate_result.rs:12
- **说明**: 重复检测和策略应用的结果

#### 结果变体

##### NotFound { url_hash, target_path }
- **说明**: 未找到重复，应该创建新任务

##### Found { task_id, reason, status }
- **说明**: 找到重复，应该重用现有任务

##### NewTask(TaskId)
- **说明**: 已创建新任务（传统变体）

##### ExistingTask { task_id, status, reason }
- **说明**: 找到现有任务并将重用（传统变体）

##### RequiresDecision { candidates, suggested_action }
- **说明**: 需要用户交互来决定

### DuplicateAction
- **位置**: src/models/duplicate_result.rs:41
- **说明**: 重复解决的建议操作

#### 操作变体

##### Resume(TaskId)
- **说明**: 恢复指定任务

##### Reuse(TaskId)
- **说明**: 重用指定任务（已完成）

##### Retry(TaskId)
- **说明**: 重试指定任务（已失败）

##### CreateNew
- **说明**: 创建新任务

## DuplicateResult 方法

### task_id()
- **位置**: src/models/duplicate_result.rs:54
- **功能**: 从任何结果变体获取任务ID
- **返回值**: `Option<TaskId>`

### is_not_found()
- **位置**: src/models/duplicate_result.rs:65
- **功能**: 检查此结果是否表示未找到重复
- **返回值**: `bool`

### is_found()
- **位置**: src/models/duplicate_result.rs:70
- **功能**: 检查此结果是否表示找到重复
- **返回值**: `bool`

### is_new_task()
- **位置**: src/models/duplicate_result.rs:75
- **功能**: 检查此结果是否表示新任务（传统）
- **返回值**: `bool`

### is_existing_task()
- **位置**: src/models/duplicate_result.rs:80
- **功能**: 检查此结果是否表示现有任务（传统）
- **返回值**: `bool`

### requires_decision()
- **位置**: src/models/duplicate_result.rs:85
- **功能**: 检查此结果是否需要用户决定
- **返回值**: `bool`

## DuplicateAction 方法

### task_id()
- **位置**: src/models/duplicate_result.rs:92
- **功能**: 获取与此操作关联的任务ID（如果有）
- **返回值**: `Option<TaskId>`

## 特征实现

### Debug, Clone, PartialEq, Eq, Serialize, Deserialize
- **说明**: 提供调试、克隆、比较和序列化支持

## 使用场景

这些类型用于：
1. 传达重复检测的结果
2. 指导后续的处理逻辑
3. 提供用户交互的选项
4. 序列化存储检测结果