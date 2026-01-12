use crate::api::{fetch_glm_quota_limit, format_reset_time, get_glm_api_config, GlmPlan, GlmPlatform};
use crate::config::InputData;
use crate::core::segments::SegmentData;
use std::collections::HashMap;

/// ANSI 重置代码
const RESET: &str = "\x1b[0m";

/// 根据百分比获取状态色（柔和色调）
fn get_status_color(percentage: f64) -> &'static str {
    if percentage <= 50.0 {
        "\x1b[38;5;114m" // 柔和绿 (256色 #114)
    } else if percentage <= 80.0 {
        "\x1b[38;5;179m" // 柔和黄/橙 (256色 #179)
    } else {
        "\x1b[38;5;167m" // 柔和红 (256色 #167)
    }
}

pub struct GlmUsageSegment;

impl GlmUsageSegment {
    pub fn new() -> Self {
        Self
    }
}

impl super::Segment for GlmUsageSegment {
    fn collect(&self, _input: &InputData) -> Option<SegmentData> {
        // 尝试从 Claude settings 获取 API 配置
        let api_config = match get_glm_api_config() {
            Some(config) => config,
            None => {
                let metadata = HashMap::new();
                return Some(SegmentData {
                    primary: "未配置 GLM".to_string(),
                    secondary: String::new(),
                    metadata,
                });
            }
        };

        let platform = api_config.platform();
        let platform_name = match platform {
            GlmPlatform::ZAI => "Z.ai",
            GlmPlatform::ZHIPU => "智谱",
        };

        // 获取 quota limit 数据（GLM 主要关注配额百分比）
        let quota_data = match fetch_glm_quota_limit(&api_config) {
            Ok(data) => data,
            Err(_) => {
                let metadata = HashMap::new();
                return Some(SegmentData {
                    primary: "⏳ 获取中...".to_string(),
                    secondary: String::new(),
                    metadata,
                });
            }
        };

        // 查找 TOKENS_LIMIT 类型的配额（5小时窗口）
        let tokens_limit = quota_data.limits.iter().find(|limit| {
            limit.limit_type == "TOKENS_LIMIT" || limit.limit_type.contains("TOKENS")
        });

        // 查找 TIME_LIMIT 类型的配额（1个月窗口，MCP usage）
        let time_limit = quota_data.limits.iter().find(|limit| {
            limit.limit_type == "TIME_LIMIT" || limit.limit_type.contains("TIME")
        });

        let mut metadata = HashMap::new();
        metadata.insert("platform".to_string(), platform_name.to_string());

        // 优先显示 TOKENS_LIMIT（主要使用指标）
        if let Some(tokens) = tokens_limit {
            let percentage = tokens.percentage;
            metadata.insert("tokens_percentage".to_string(), format!("{:.1}", percentage));

            // 识别套餐类型（仅记录在 metadata 中）
            if let Some(usage) = tokens.usage {
                let detected_plan = GlmPlan::from_token_usage(usage);
                metadata.insert("plan".to_string(), detected_plan.name().to_string());
            }

            // 生成进度条（10格）
            let bar_length = 10;
            let filled = ((percentage / 100.0) * bar_length as f64).round() as usize;
            let empty = bar_length - filled;

            let status_color = get_status_color(percentage);
            let progress_bar = format!(
                "{}{}{}{}",
                status_color,
                "▓".repeat(filled),
                "░".repeat(empty),
                RESET
            );

            // 格式化重置时间（如果存在）
            let reset_time_str = if let Some(reset_ts) = tokens.next_reset_time {
                metadata.insert("next_reset_time".to_string(), reset_ts.to_string());
                let formatted = format_reset_time(reset_ts);
                format!(" 🔄 {}", formatted)
            } else {
                String::new()
            };

            // 构建显示文本：进度条 百分比 重置时间
            let primary = format!(
                "{} {}{}",
                progress_bar,
                format_percentage(percentage),
                reset_time_str
            );

            // 如果有 TIME_LIMIT，显示在 secondary
            let secondary = if let Some(time) = time_limit {
                metadata.insert("time_percentage".to_string(), format!("{:.1}", time.percentage));
                format!("MCP {}", format_percentage(time.percentage))
            } else {
                String::new()
            };

            return Some(SegmentData {
                primary,
                secondary,
                metadata,
            });
        }

        // 如果没有 TOKENS_LIMIT，但有 TIME_LIMIT
        if let Some(time) = time_limit {
            let percentage = time.percentage;
            metadata.insert("time_percentage".to_string(), format!("{:.1}", percentage));

            let bar_length = 10;
            let filled = ((percentage / 100.0) * bar_length as f64).round() as usize;
            let empty = bar_length - filled;

            let status_color = get_status_color(percentage);
            let progress_bar = format!(
                "{}{}{}{}",
                status_color,
                "▓".repeat(filled),
                "░".repeat(empty),
                RESET
            );

            return Some(SegmentData {
                primary: format!(
                    "{} MCP {} {}",
                    platform_name,
                    progress_bar,
                    format_percentage(percentage)
                ),
                secondary: String::new(),
                metadata,
            });
        }

        // 没有任何配额数据
        Some(SegmentData {
            primary: format!("{} 暂无数据", platform_name),
            secondary: String::new(),
            metadata,
        })
    }

    fn id(&self) -> crate::config::SegmentId {
        crate::config::SegmentId::GlmUsage
    }
}

/// 格式化百分比
fn format_percentage(percentage: f64) -> String {
    format!("{:.0}%", percentage)
}
