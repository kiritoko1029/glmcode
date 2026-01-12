use crate::api::{fetch_glm_model_performance, fetch_glm_quota_limit, get_glm_api_config, GlmPlan};
use crate::config::InputData;
use crate::core::segments::SegmentData;
use std::collections::HashMap;

/// ANSI 重置代码
const RESET: &str = "\x1b[0m";

/// 生成速度曲线 sparkline
fn generate_sparkline(speeds: &[f64], _width: usize) -> String {
    if speeds.is_empty() {
        return "N/A".to_string();
    }

    let max_speed = speeds.iter().fold(0.0f64, |acc, &v| acc.max(v));
    let min_speed = speeds.iter().fold(f64::INFINITY, |acc, &v| acc.min(v));

    // 使用 8 级 sparkline 字符
    let bars: Vec<String> = speeds
        .iter()
        .map(|&speed| {
            if max_speed == min_speed {
                "▄".to_string()
            } else {
                let normalized = (speed - min_speed) / (max_speed - min_speed);
                let chars = [" ", "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
                let level = (normalized * (chars.len() - 1) as f64).round() as usize;
                chars[level.min(chars.len() - 1)].to_string()
            }
        })
        .collect();

    bars.join("")
}

pub struct DecodeSpeedSegment;

impl DecodeSpeedSegment {
    pub fn new() -> Self {
        Self
    }
}

impl super::Segment for DecodeSpeedSegment {
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

        // 获取套餐类型
        let plan = match fetch_glm_quota_limit(&api_config) {
            Ok(quota_data) => {
                // 查找 TOKENS_LIMIT 类型的配额
                quota_data.limits.iter().find_map(|limit| {
                    if limit.limit_type == "TOKENS_LIMIT" || limit.limit_type.contains("TOKENS") {
                        limit.usage.map(|usage| GlmPlan::from_token_usage(usage))
                    } else {
                        None
                    }
                }).unwrap_or(GlmPlan::Unknown)
            }
            Err(_) => GlmPlan::Unknown,
        };

        // 获取模型性能数据（最近5小时）
        let performance_data = match fetch_glm_model_performance(&api_config, 5) {
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

        // 根据套餐类型选择对应的解码速度
        let speeds = match plan {
            GlmPlan::Lite => &performance_data.lite_decode_speed,
            GlmPlan::Pro | GlmPlan::Max => &performance_data.pro_max_decode_speed,
            GlmPlan::Unknown => &performance_data.pro_max_decode_speed, // 默认使用 Pro/Max
        };

        // 只取最近5个数据点（如果有的话）
        let recent_speeds: Vec<f64> = speeds.iter().rev().take(5).cloned().collect();
        let recent_speeds_rev: Vec<f64> = recent_speeds.iter().cloned().rev().collect();

        // 获取当前速度（最后一个数据点）
        let current_speed = if recent_speeds_rev.is_empty() {
            0.0
        } else {
            *recent_speeds_rev.last().unwrap()
        };

        // 生成 sparkline 曲线
        let sparkline = generate_sparkline(&recent_speeds_rev, 5);

        // 计算速度变化趋势
        let trend = if recent_speeds_rev.len() >= 2 {
            let first = *recent_speeds_rev.first().unwrap();
            let last = *recent_speeds_rev.last().unwrap();
            if last > first * 1.05 {
                "↗"  // 上升超过 5%
            } else if last < first * 0.95 {
                "↘"  // 下降超过 5%
            } else {
                "→"  // 平稳
            }
        } else {
            "→"
        };

        let mut metadata = HashMap::new();
        metadata.insert("plan".to_string(), plan.name().to_string());
        metadata.insert("current_speed".to_string(), format!("{:.1}", current_speed));
        metadata.insert("sparkline".to_string(), sparkline.clone());

        // 构建显示文本：套餐 趋势 速度t/s 曲线
        let primary = format!(
            "{} {}{}t/s {}{}",
            plan.name(),
            trend,
            format_speed(current_speed),
            "\x1b[38;5;214m",  // 橙色
            sparkline
        );

        let secondary = String::new();

        Some(SegmentData {
            primary: format!("{}{}", primary, RESET),
            secondary,
            metadata,
        })
    }

    fn id(&self) -> crate::config::SegmentId {
        crate::config::SegmentId::DecodeSpeed
    }
}

/// 格式化速度显示
fn format_speed(speed: f64) -> String {
    if speed >= 1000.0 {
        format!("{:.1}k", speed / 1000.0)
    } else {
        format!("{:.0}", speed)
    }
}
