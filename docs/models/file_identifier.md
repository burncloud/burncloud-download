# models/file_identifier.rs - 文件标识符

## 结构体

### FileIdentifier
- **位置**: src/models/file_identifier.rs:12
- **说明**: 用于重复检测的组合键，基于标准化URL哈希和目标路径标识文件

#### 字段
- `url_hash: String` - 标准化后的URL哈希值
- `target_path: PathBuf` - 目标文件路径
- `file_size: Option<u64>` - 文件大小（可选）

## 方法实现

### new(url, target_path, file_size)
- **位置**: src/models/file_identifier.rs:20
- **功能**: 使用标准化URL哈希创建新的FileIdentifier
- **参数**:
  - `url: &str` - 原始URL
  - `target_path: &Path` - 目标路径
  - `file_size: Option<u64>` - 文件大小（可选）
- **返回值**: `Self`
- **说明**: 对URL进行标准化处理并生成哈希，如果标准化失败则使用原始URL的Blake3哈希作为回退

### matches_task(task)
- **位置**: src/models/file_identifier.rs:36
- **功能**: 检查此标识符是否匹配下载任务
- **参数**: `task: &T` - 实现了HasUrlHashAndPath特征的任务
- **返回值**: `bool` - 是否匹配
- **说明**: 比较URL哈希和目标路径是否都相同

## 特征定义

### HasUrlHashAndPath
- **位置**: src/models/file_identifier.rs:45
- **说明**: 用于重复检测的类型特征，要求类型具有url_hash和target_path

#### 必需方法

##### url_hash()
- **位置**: src/models/file_identifier.rs:46
- **功能**: 返回URL哈希的引用
- **返回值**: `&str`

##### target_path()
- **位置**: src/models/file_identifier.rs:47
- **功能**: 返回目标路径的引用
- **返回值**: `&Path`

## 特征实现

### Debug, Clone, PartialEq, Eq, Hash
- **位置**: src/models/file_identifier.rs:11
- **说明**: 提供调试、克隆、比较和哈希支持，使其可以用作HashMap的键

## 重复检测机制

FileIdentifier提供了基于以下因素的重复检测：

1. **URL标准化**: 使用`process_url_for_storage`对URL进行标准化处理
2. **哈希生成**: 使用Blake3算法生成URL哈希，确保一致性
3. **路径匹配**: 精确匹配目标文件路径
4. **回退机制**: 如果URL标准化失败，使用原始URL的哈希作为回退

## 使用场景

- 检测相同URL到相同路径的重复下载
- 作为数据库查询的键值
- 在内存中快速比较和查找重复任务
- 支持哈希映射和集合操作

## 依赖项

- `std::path::{Path, PathBuf}` - 路径处理
- `crate::utils::url_normalization::process_url_for_storage` - URL标准化
- `blake3` - 哈希算法