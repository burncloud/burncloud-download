# models/task_status.rs - 扩展任务状态

## 枚举类型

### TaskStatus
- **位置**: src/models/task_status.rs:12
- **说明**: 扩展的任务状态，包含重复检测状态，同时保持与现有DownloadStatus的兼容性

#### 状态变体

##### Waiting
- **说明**: 任务等待开始

##### Downloading
- **说明**: 任务正在下载

##### Paused
- **说明**: 任务已暂停

##### Completed
- **说明**: 任务成功完成

##### Failed(String)
- **说明**: 任务失败，包含错误消息

##### Duplicate(TaskId)
- **说明**: 任务是另一个任务的重复

## 方法实现

### can_transition_to_duplicate()
- **位置**: src/models/task_status.rs:29
- **功能**: 检查此状态是否可以转换为Duplicate状态
- **返回值**: `bool`
- **说明**: 只有Waiting、Paused、Failed状态可以转换为Duplicate

### to_download_status()
- **位置**: src/models/task_status.rs:38
- **功能**: 转换为基础DownloadStatus以保持兼容性
- **返回值**: `crate::types::DownloadStatus`

### from_download_status(status)
- **位置**: src/models/task_status.rs:54
- **功能**: 从基础DownloadStatus创建
- **参数**: `status: crate::types::DownloadStatus`
- **返回值**: `Self`

## 验证器

### TaskValidator
- **位置**: src/models/task_status.rs:66
- **说明**: 任务相关数据的验证工具

#### 验证方法

##### validate_url_hash(url_hash)
- **位置**: src/models/task_status.rs:70
- **功能**: 验证URL哈希格式是否符合数据库存储要求
- **参数**: `url_hash: &str`
- **返回值**: `Result<(), TaskValidationError>`

##### validate_task_id(task_id)
- **位置**: src/models/task_status.rs:81
- **功能**: 验证任务ID不为空或默认值
- **参数**: `task_id: &TaskId`
- **返回值**: `Result<(), TaskValidationError>`

##### validate_status_transition(from, to)
- **位置**: src/models/task_status.rs:92
- **功能**: 验证任务状态转换是否有效
- **参数**:
  - `from: &TaskStatus` - 原状态
  - `to: &TaskStatus` - 目标状态
- **返回值**: `Result<(), TaskValidationError>`

## 错误类型

### TaskValidationError
- **位置**: src/models/task_status.rs:133
- **说明**: 任务相关操作的验证错误

#### 错误变体

##### InvalidUrlHash { hash, expected_format }
- **说明**: 无效的URL哈希格式

##### InvalidTaskId { reason }
- **说明**: 无效的任务ID

##### InvalidStatusTransition { from, to, reason }
- **说明**: 无效的状态转换