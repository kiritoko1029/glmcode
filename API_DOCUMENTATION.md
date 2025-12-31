# Usage Monitor API 接口文档

## 概述

Usage Monitor API 用于查询模型使用量、工具使用量和配额限制信息。目前支持两个平台：
- **Z.ai 平台**: `https://api.z.ai`
- **智谱平台（ZHIPU）**: `https://open.bigmodel.cn` / `https://dev.bigmodel.cn`

## 认证方式

所有 API 请求需要在 HTTP Header 中包含以下认证信息：

```http
Authorization: {YOUR_AUTH_TOKEN}
Accept-Language: en-US,en
Content-Type: application/json
```

**环境变量配置**：
```bash
export ANTHROPIC_AUTH_TOKEN="your-token-here"
export ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"
```

## 基础 URL

### Z.ai 平台
```
https://api.z.ai
```

### 智谱平台
```
https://open.bigmodel.cn
```
或
```
https://dev.bigmodel.cn
```

## API 端点

---

## 1. 查询模型使用量

### 端点

```
GET /api/monitor/usage/model-usage
```

### 描述

查询指定时间范围内的模型使用量统计数据。

### 请求参数

| 参数名 | 类型 | 必填 | 描述 | 示例 |
|--------|------|------|------|------|
| startTime | string | 是 | 查询开始时间 | `2025-01-01 12:00:00` |
| endTime | string | 是 | 查询结束时间 | `2025-01-02 12:59:59` |

**时间格式**: `yyyy-MM-dd HH:mm:ss`

**默认时间窗口**: 从昨天当前小时（HH:00:00）到今天当前小时结束（HH:59:59）

### 请求示例

```bash
curl -X GET "https://open.bigmodel.cn/api/monitor/usage/model-usage?startTime=2025-01-01%2012%3A00%3A00&endTime=2025-01-02%2012%3A59%3A59" \
  -H "Authorization: " \
  -H "Accept-Language: en-US,en" \
  -H "Content-Type: application/json"
```

### 响应示例

```json
{
  "data": {
    "totalTokens": 150000,
    "totalRequests": 245,
    "models": [
      {
        "modelName": "claude-sonnet-4-20250514",
        "tokens": 120000,
        "requests": 200
      },
      {
        "modelName": "claude-opus-4-20250514",
        "tokens": 30000,
        "requests": 45
      }
    ]
  }
}
```

---

## 2. 查询工具使用量

### 端点

```
GET /api/monitor/usage/tool-usage
```

### 描述

查询指定时间范围内的工具（Tools/MCP）使用量统计数据。

### 请求参数

| 参数名 | 类型 | 必填 | 描述 | 示例 |
|--------|------|------|------|------|
| startTime | string | 是 | 查询开始时间 | `2025-01-01 12:00:00` |
| endTime | string | 是 | 查询结束时间 | `2025-01-02 12:59:59` |

**时间格式**: `yyyy-MM-dd HH:mm:ss`

### 请求示例

```bash
curl -X GET "https://open.bigmodel.cn/api/monitor/usage/tool-usage?startTime=2025-01-01%2012%3A00%3A00&endTime=2025-01-02%2012%3A59%3A59" \
  -H "Authorization: " \
  -H "Accept-Language: en-US,en" \
  -H "Content-Type: application/json"
```

### 响应示例

```json
{
  "data": {
    "totalCalls": 1250,
    "tools": [
      {
        "toolName": "file_search",
        "calls": 800
      },
      {
        "toolName": "web_search",
        "calls": 450
      }
    ]
  }
}
```

---

## 3. 查询配额限制

### 端点

```
GET /api/monitor/usage/quota/limit
```

### 描述

查询当前账户的配额限制和使用情况。

### 请求参数

无需查询参数。

### 请求示例

```bash
curl -X GET "https://open.bigmodel.cn/api/monitor/usage/quota/limit" \
  -H "Authorization: " \
  -H "Accept-Language: en-US,en" \
  -H "Content-Type: application/json"
```

### 响应示例（原始）

```json
{
  "code": 200,
  "msg": "Operation successful",
  "data": {
    "limits": [
      {
        "type": "TIME_LIMIT",
        "unit": 5,
        "number": 1,
        "usage": 1000,
        "currentValue": 0,
        "remaining": 1000,
        "percentage": 0,
        "usageDetails": [
          {
            "modelCode": "search-prime",
            "usage": 0
          },
          {
            "modelCode": "web-reader",
            "usage": 0
          },
          {
            "modelCode": "zread",
            "usage": 0
          }
        ]
      },
      {
        "type": "TOKENS_LIMIT",
        "unit": 3,
        "number": 5,
        "usage": 200000000,
        "currentValue": 18366001,
        "remaining": 181633999,
        "percentage": 9,
        "nextResetTime": 1767163875150
      }
    ]
  },
  "success": true
}
```

### 响应示例（处理后）

脚本会自动处理响应数据，转换为更易读的格式，并自动识别套餐类型：

```json
{
  "limits": [
    {
      "type": "MCP 用量 (1 个月)",
      "unit": 5,
      "number": 1,
      "usage": 1000,
      "currentUsage": 0,
      "total": 1000,
      "remaining": 1000,
      "percentage": 0,
      "usageDetails": [
        {
          "modelCode": "search-prime",
          "usage": 0
        },
        {
          "modelCode": "web-reader",
          "usage": 0
        },
        {
          "modelCode": "zread",
          "usage": 0
        }
      ]
    },
    {
      "type": "Token 用量 (5 小时)",
      "unit": 3,
      "number": 5,
      "plan": "Pro",
      "planName": "套餐: Pro",
      "usage": 200000000,
      "usageFormatted": "200,000,000",
      "currentValue": 18366001,
      "currentValueFormatted": "18,366,001",
      "remaining": 181633999,
      "remainingFormatted": "181,633,999",
      "percentage": 9,
      "nextResetTime": 1767163875150,
      "nextResetTimeReadable": "2025/12/31 12:04:35"
    }
  ]
}
```

### 套餐类型识别规则

脚本会根据 `TOKENS_LIMIT` 的 `usage` 值自动识别套餐类型：

| 套餐类型 | Token 配额 (usage) | 与 Pro 的关系 | 说明 |
|---------|-------------------|--------------|------|
| **Lite** | 40,000,000 (4千万) | Pro 的 1/5 | 入门套餐 |
| **Pro** | 200,000,000 (2亿) | 基准 | 专业套餐 |
| **Max** | 800,000,000 (8亿) | Pro 的 4倍 | 旗舰套餐 |
| **Unknown** | 其他值 | - | 未知或自定义套餐 |

**识别逻辑**：
```javascript
const PRO_LIMIT = 200000000;  // 2亿
const LITE_LIMIT = PRO_LIMIT / 5;  // 4千万
const MAX_LIMIT = PRO_LIMIT * 4;   // 8亿
```

### 配额类型说明

| 配额类型 | 原始标识 | 显示名称格式 | 重置周期 | 说明 |
|---------|---------|-------------|---------|------|
| Token 配额 | `TOKENS_LIMIT` | `Token 用量 ({number} {unit})` | unit=3: 小时 / unit=4: 天 | Token 使用量限制 |
| 时长配额 | `TIME_LIMIT` | `MCP 用量 ({number} {unit})` | unit=5: 个月 / unit=4: 天 | MCP 工具使用时长限制 |

**unit 字段说明**：
- `3` = 小时
- `4` = 天
- `5` = 个月

---

## 响应字段说明

### 通用响应结构

智谱平台的 API 响应遵循以下通用结构：

```json
{
  "code": 200,
  "msg": "Operation successful",
  "data": {
    // 具体业务数据
  },
  "success": true
}
```

### 配额限制响应字段

#### 原始响应字段

| 字段名 | 类型 | 描述 | 适用配额类型 |
|--------|------|------|-------------|
| type | string | 配额类型标识 (`TOKENS_LIMIT` / `TIME_LIMIT`) | 全部 |
| unit | number | 时间单位 (3=小时, 4=天, 5=个月) | 全部 |
| number | number | 时间数量（与 unit 配合使用） | 全部 |
| usage | number | 总配额值 | 全部 |
| currentValue | number | 当前已使用量 | 全部 |
| remaining | number | 剩余配额 | 全部 |
| percentage | number | 使用百分比 (0-100) | 全部 |
| nextResetTime | number | 下次重置时间（Unix 时间戳，毫秒） | TOKENS_LIMIT |
| usageDetails | array | MCP 工具使用详情 | TIME_LIMIT |

#### 处理后新增字段

| 字段名 | 类型 | 描述 | 适用配额类型 |
|--------|------|------|-------------|
| plan | string | 套餐类型 (`Lite` / `Pro` / `Max` / `Unknown`) | TOKENS_LIMIT |
| planName | string | 套餐名称（格式：`套餐: {plan}`) | TOKENS_LIMIT |
| usageFormatted | string | 格式化的总配额（带千分位） | TOKENS_LIMIT |
| currentValueFormatted | string | 格式化的当前使用量（带千分位） | TOKENS_LIMIT |
| remainingFormatted | string | 格式化的剩余配额（带千分位） | TOKENS_LIMIT |
| nextResetTimeReadable | string | 可读的重置时间（北京时间） | TOKENS_LIMIT |
| currentUsage | number | 当前使用量（别名，同 currentValue） | TIME_LIMIT |
| total | number | 总配额（别名，同 usage） | TIME_LIMIT |

---

## 错误处理

### HTTP 状态码

| 状态码 | 说明 |
|--------|------|
| 200 | 请求成功 |
| 400 | 请求参数错误 |
| 401 | 认证失败（Token 无效或过期） |
| 403 | 权限不足 |
| 404 | 端点不存在 |
| 429 | 请求频率超限 |
| 500 | 服务器内部错误 |

### 错误响应示例

```json
{
  "error": {
    "code": "INVALID_TOKEN",
    "message": "Authentication token is invalid or expired",
    "details": "Please check your ANTHROPIC_AUTH_TOKEN"
  }
}
```

---

## 使用示例

### Node.js 脚本使用

```bash
# 设置环境变量
export ANTHROPIC_AUTH_TOKEN="your-token-here"
export ANTHROPIC_BASE_URL="https://open.bigmodel.cn/api/anthropic"

# 运行查询脚本
node query-usage.mjs
```

### 输出示例

```
Platform: ZHIPU

Model usage data:

Full API Response:
{
  "code": 200,
  "msg": "Operation successful",
  "data": {
    "totalTokens": 150000,
    "totalRequests": 245
  },
  "success": true
}

Processed Data:
{
  "totalTokens": 150000,
  "totalRequests": 245
}

Tool usage data:

Full API Response:
{
  "code": 200,
  "msg": "Operation successful",
  "data": {
    "totalCalls": 1250
  },
  "success": true
}

Processed Data:
{
  "totalCalls": 1250
}

Quota limit data:

Full API Response:
{
  "code": 200,
  "msg": "Operation successful",
  "data": {
    "limits": [
      {
        "type": "TIME_LIMIT",
        "unit": 5,
        "number": 1,
        "usage": 1000,
        "currentValue": 0,
        "remaining": 1000,
        "percentage": 0,
        "usageDetails": [...]
      },
      {
        "type": "TOKENS_LIMIT",
        "unit": 3,
        "number": 5,
        "usage": 200000000,
        "currentValue": 18366001,
        "remaining": 181633999,
        "percentage": 9,
        "nextResetTime": 1767163875150
      }
    ]
  },
  "success": true
}

Processed Data:
{
  "limits": [
    {
      "type": "MCP 用量 (1 个月)",
      "unit": 5,
      "number": 1,
      "usage": 1000,
      "currentUsage": 0,
      "total": 1000,
      "remaining": 1000,
      "percentage": 0,
      "usageDetails": [...]
    },
    {
      "type": "Token 用量 (5 小时)",
      "unit": 3,
      "number": 5,
      "plan": "Pro",
      "planName": "套餐: Pro",
      "usage": 200000000,
      "usageFormatted": "200,000,000",
      "currentValue": 18366001,
      "currentValueFormatted": "18,366,001",
      "remaining": 181633999,
      "remainingFormatted": "181,633,999",
      "percentage": 9,
      "nextResetTime": 1767163875150,
      "nextResetTimeReadable": "2025/12/31 12:04:35"
    }
  ]
}
```

**输出说明**：
- ✅ **套餐自动识别**：根据 Token 配额自动识别为 Pro 套餐
- ✅ **数字格式化**：大数字使用千分位分隔符，易于阅读
- ✅ **时间本地化**：重置时间转换为北京时间可读格式
- ✅ **类型中文化**：配额类型转换为中文描述

---

## 注意事项

1. **时间格式**：所有时间参数使用 `yyyy-MM-dd HH:mm:ss` 格式
2. **URL 编码**：查询参数中的空格和特殊字符需要正确编码
3. **时区**：`nextResetTimeReadable` 字段使用北京时间（Asia/Shanghai）
4. **令牌安全**：请勿将 `ANTHROPIC_AUTH_TOKEN` 硬编码在代码中或提交到版本控制系统
5. **时间窗口**：默认查询窗口为 25 小时（昨天当前小时到今天当前小时）
6. **响应处理**：配额限制接口的响应会经过后处理，将类型标识转换为易读名称
7. **套餐识别**：脚本会根据 Token 配额自动识别套餐类型（Lite/Pro/Max/Unknown）
8. **数字格式化**：大数字会自动添加千分位分隔符，便于阅读

---

## 更新日志

### v1.1.0 (2025-12-31)
- ✨ 新增套餐类型自动识别功能（Lite/Pro/Max/Unknown）
- ✨ 新增数字千分位格式化显示
- 📝 更新响应字段说明，支持新的 API 格式
- 📝 更新配额类型说明，支持动态时间单位
- 🐛 修复时间单位解析逻辑

### v1.0.0 (2025-01-01)
- 初始版本
- 支持模型使用量查询
- 支持工具使用量查询
- 支持配额限制查询
- 支持 Z.ai 和智谱双平台

---

## 技术支持

如有疑问或需要帮助，请联系：
- **Z.ai 平台**: support@z.ai
- **智谱平台**: support@bigmodel.cn
