use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// GLM API 配置
#[derive(Debug, Clone)]
pub struct GlmApiConfig {
    pub base_url: String,
    pub auth_token: String,
}

/// GLM 平台类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GlmPlatform {
    ZAI,
    ZHIPU,
}

/// GLM 套餐类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GlmPlan {
    Lite,
    Pro,
    Max,
    Unknown,
}

impl GlmPlan {
    /// 从 Token usage 配额识别套餐类型
    pub fn from_token_usage(usage: f64) -> Self {
        const PRO_LIMIT: f64 = 200_000_000.0;  // 2亿
        const LITE_LIMIT: f64 = PRO_LIMIT / 5.0;  // 4千万
        const MAX_LIMIT: f64 = PRO_LIMIT * 4.0;   // 8亿

        if (usage - LITE_LIMIT).abs() < 1.0 {
            Self::Lite
        } else if (usage - PRO_LIMIT).abs() < 1.0 {
            Self::Pro
        } else if (usage - MAX_LIMIT).abs() < 1.0 {
            Self::Max
        } else {
            Self::Unknown
        }
    }

    /// 获取套餐名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Lite => "Lite",
            Self::Pro => "Pro",
            Self::Max => "Max",
            Self::Unknown => "Unknown",
        }
    }

    /// 获取套餐显示名称（不带 emoji）
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Lite => "Lite",
            Self::Pro => "Pro",
            Self::Max => "Max",
            Self::Unknown => "Unknown",
        }
    }
}

impl GlmApiConfig {
    /// 从配置创建 GLM API 配置
    pub fn new(base_url: String, auth_token: String) -> Self {
        Self { base_url, auth_token }
    }

    /// 判断平台类型
    pub fn platform(&self) -> GlmPlatform {
        if self.base_url.contains("api.z.ai") {
            GlmPlatform::ZAI
        } else if self.base_url.contains("bigmodel.cn") {
            GlmPlatform::ZHIPU
        } else {
            // 默认为 ZHIPU
            GlmPlatform::ZHIPU
        }
    }

    /// 构建 quota limit URL
    pub fn quota_limit_url(&self) -> String {
        let parsed = url::Url::parse(&self.base_url).unwrap();
        let base = format!("{}://{}", parsed.scheme(), parsed.host().unwrap());
        format!("{}/api/monitor/usage/quota/limit", base)
    }

    /// 构建模型性能 URL
    pub fn model_performance_url(&self) -> String {
        let parsed = url::Url::parse(&self.base_url).unwrap();
        let base = format!("{}://{}", parsed.scheme(), parsed.host().unwrap());
        format!("{}/api/monitor/usage/model-performance", base)
    }
}

/// GLM quota limit 数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GlmQuotaLimit {
    pub limits: Vec<QuotaLimitItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QuotaLimitItem {
    #[serde(rename = "type")]
    pub limit_type: String,
    pub percentage: f64,
    #[serde(default)]
    pub current_value: Option<f64>,
    #[serde(default)]
    pub usage: Option<f64>,
    #[serde(default)]
    pub usage_details: Option<serde_json::Value>,
    #[serde(rename = "nextResetTime", default)]
    pub next_reset_time: Option<i64>,
}

/// Claude settings.json 结构
#[derive(Debug, Deserialize)]
struct ClaudeSettings {
    env: Option<ClaudeEnv>,
}

#[derive(Debug, Deserialize)]
struct ClaudeEnv {
    #[serde(rename = "ANTHROPIC_AUTH_TOKEN")]
    auth_token: Option<String>,
    #[serde(rename = "ANTHROPIC_BASE_URL")]
    base_url: Option<String>,
}

/// 获取 Claude settings.json 路径
fn get_claude_settings_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".claude").join("settings.json"))
}

/// 从 Claude settings.json 读取 GLM API 配置
pub fn get_glm_api_config() -> Option<GlmApiConfig> {
    let settings_path = get_claude_settings_path()?;

    if !settings_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(settings_path).ok()?;
    let settings: ClaudeSettings = serde_json::from_str(&content).ok()?;

    let env = settings.env?;

    // 检查是否是 Z.ai 或 ZHIPU 平台
    let base_url = env.base_url?;
    if !base_url.contains("api.z.ai") && !base_url.contains("bigmodel.cn") {
        return None;
    }

    let auth_token = env.auth_token?;

    Some(GlmApiConfig::new(base_url, auth_token))
}

/// 查询 GLM quota limit
pub fn fetch_glm_quota_limit(
    config: &GlmApiConfig,
) -> Result<GlmQuotaLimit, Box<dyn std::error::Error>> {
    use reqwest::blocking::Client;
    use std::time::Duration;

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("glmcode/1.0.0")
        .build()?;

    let response = client
        .get(&config.quota_limit_url())
        .header("Authorization", &config.auth_token)
        .header("Accept-Language", "en-US,en")
        .header("Content-Type", "application/json")
        .send()?;

    if !response.status().is_success() {
        return Err(format!("GLM Quota API request failed: {}", response.status()).into());
    }

    let response_text = response.text()?;
    let json: serde_json::Value = serde_json::from_str(&response_text)?;

    // Debug: 输出原始 JSON 响应（仅在开发模式下）
    #[cfg(debug_assertions)]
    eprintln!("[DEBUG] GLM API Response: {}", serde_json::to_string_pretty(&json).unwrap_or_default());

    let quota_data = extract_quota_limit_data(&json)?;

    Ok(quota_data)
}

/// 从 JSON 响应中提取 quota limit 数据
fn extract_quota_limit_data(
    json: &serde_json::Value,
) -> Result<GlmQuotaLimit, Box<dyn std::error::Error>> {
    // 尝试获取顶层数据对象：json.data 或 json 本身
    let data_obj = json.get("data").unwrap_or(json);

    // 从数据对象中提取 limits 数组
    let limits_value = data_obj
        .get("limits")
        .ok_or("Missing 'limits' field in response")?;

    // 解析 limits 数组
    let limits: Vec<QuotaLimitItem> = serde_json::from_value(limits_value.clone())?;

    Ok(GlmQuotaLimit { limits })
}

/// 格式化重置时间为相对时间显示（如 "3h 40m", "25m", "2d 5h"）
pub fn format_reset_time(reset_timestamp_ms: i64) -> String {
    // 获取当前时间（毫秒）
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    let diff_ms = reset_timestamp_ms - now_ms;

    // 如果时间已过，返回 "expired"
    if diff_ms <= 0 {
        return "expired".to_string();
    }

    let diff_sec = diff_ms / 1000;
    let diff_min = diff_sec / 60;
    let diff_hour = diff_min / 60;
    let diff_day = diff_hour / 24;

    // 根据时间范围选择合适的格式
    if diff_day > 0 {
        let hours = diff_hour % 24;
        if hours > 0 {
            format!("{}d {}h", diff_day, hours)
        } else {
            format!("{}d", diff_day)
        }
    } else if diff_hour > 0 {
        let minutes = diff_min % 60;
        if minutes > 0 {
            format!("{}h {}m", diff_hour, minutes)
        } else {
            format!("{}h", diff_hour)
        }
    } else if diff_min > 0 {
        format!("{}m", diff_min)
    } else {
        format!("{}s", diff_sec)
    }
}

/// GLM 模型性能数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GlmModelPerformance {
    pub x_time: Vec<String>,
    pub lite_decode_speed: Vec<f64>,
    pub pro_max_decode_speed: Vec<f64>,
    pub lite_success_rate: Vec<f64>,
    pub pro_max_success_rate: Vec<f64>,
}

/// 查询 GLM 模型性能
pub fn fetch_glm_model_performance(
    config: &GlmApiConfig,
    hours: i64,
) -> Result<GlmModelPerformance, Box<dyn std::error::Error>> {
    use reqwest::blocking::Client;
    use std::time::Duration;

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("glmcode/1.0.0")
        .build()?;

    // 计算时间范围：过去 N 小时
    let now = chrono::Local::now();
    let end_time = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let start_time = (now - chrono::Duration::hours(hours))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    let url = format!(
        "{}?startTime={}&endTime={}",
        config.model_performance_url(),
        urlencoding::encode(&start_time),
        urlencoding::encode(&end_time)
    );

    let response = client
        .get(&url)
        .header("Authorization", &config.auth_token)
        .header("Accept-Language", "en-US,en")
        .header("Content-Type", "application/json")
        .send()?;

    if !response.status().is_success() {
        return Err(format!("GLM Model Performance API request failed: {}", response.status()).into());
    }

    let response_text = response.text()?;
    let json: serde_json::Value = serde_json::from_str(&response_text)?;

    // Debug: 输出原始 JSON 响应（仅在开发模式下）
    #[cfg(debug_assertions)]
    eprintln!("[DEBUG] GLM Model Performance API Response: {}", serde_json::to_string_pretty(&json).unwrap_or_default());

    let performance_data = extract_model_performance_data(&json)?;

    Ok(performance_data)
}

/// 从 JSON 响应中提取模型性能数据
fn extract_model_performance_data(
    json: &serde_json::Value,
) -> Result<GlmModelPerformance, Box<dyn std::error::Error>> {
    // 尝试获取顶层数据对象：json.data 或 json 本身
    let data_obj = json.get("data").unwrap_or(json);

    // 解析各个字段
    let x_time: Vec<String> = serde_json::from_value(
        data_obj.get("x_time").ok_or("Missing 'x_time' field")?.clone()
    )?;

    let lite_decode_speed: Vec<f64> = serde_json::from_value(
        data_obj.get("liteDecodeSpeed").ok_or("Missing 'liteDecodeSpeed' field")?.clone()
    )?;

    let pro_max_decode_speed: Vec<f64> = serde_json::from_value(
        data_obj.get("proMaxDecodeSpeed").ok_or("Missing 'proMaxDecodeSpeed' field")?.clone()
    )?;

    let lite_success_rate: Vec<f64> = serde_json::from_value(
        data_obj.get("liteSuccessRate").ok_or("Missing 'liteSuccessRate' field")?.clone()
    )?;

    let pro_max_success_rate: Vec<f64> = serde_json::from_value(
        data_obj.get("proMaxSuccessRate").ok_or("Missing 'proMaxSuccessRate' field")?.clone()
    )?;

    Ok(GlmModelPerformance {
        x_time,
        lite_decode_speed,
        pro_max_decode_speed,
        lite_success_rate,
        pro_max_success_rate,
    })
}
