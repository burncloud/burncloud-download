# models/duplicate_reason.rs - 重复检测原因

## 枚举类型

### DuplicateReason
- **位置**: src/models/duplicate_reason.rs:9
- **说明**: 说明为什么下载被识别为重复的原因

#### 原因变体

##### ExactMatch
- **说明**: 精确匹配 - 相同的URL哈希和目标路径

##### UrlAndPath
- **说明**: 相同的URL和目标路径（传统变体）

##### FileContent
- **说明**: 相同的文件内容哈希

##### SimilarUrl
- **说明**: 标准化后的相似URL

##### Filename
- **说明**: 目标目录中的相同文件名

##### PolicyAllowed
- **说明**: 基于策略的允许（例如，重新下载已完成的文件）

## 方法实现

### description()
- **位置**: src/models/duplicate_reason.rs:26
- **功能**: 获取重复原因的人类可读描述
- **返回值**: `&'static str`
- **说明**: 为每个原因变体提供清晰的文本描述

### priority()
- **位置**: src/models/duplicate_reason.rs:38
- **功能**: 获取此重复原因的优先级（数字越小优先级越高）
- **返回值**: `u8`
- **说明**: 优先级排序：
  - ExactMatch: 0（最高优先级）
  - UrlAndPath: 1
  - FileContent: 2
  - SimilarUrl: 3
  - Filename: 4
  - PolicyAllowed: 5（最低优先级）

### is_strong_match()
- **位置**: src/models/duplicate_reason.rs:50
- **功能**: 检查此原因是否表示强重复匹配
- **返回值**: `bool`
- **说明**: 强匹配包括ExactMatch、UrlAndPath和FileContent

## 特征实现

### Display
- **位置**: src/models/duplicate_reason.rs:59
- **说明**: 实现Display特征，使用description()方法

### Debug, Clone, PartialEq, Eq, Serialize, Deserialize
- **位置**: src/models/duplicate_reason.rs:8
- **说明**: 提供调试、克隆、比较和序列化支持

## 用途

DuplicateReason用于：
1. 记录重复检测的具体原因
2. 按优先级排序重复候选者
3. 向用户解释为什么检测到重复
4. 在日志中提供详细的诊断信息
5. 帮助调试重复检测逻辑

## 优先级系统

优先级系统确保在有多个重复候选者时，系统优先选择最可靠的匹配：
- 精确哈希匹配具有最高优先级
- URL和路径匹配次之
- 内容哈希匹配也很重要
- URL相似性匹配优先级较低
- 仅文件名匹配优先级最低