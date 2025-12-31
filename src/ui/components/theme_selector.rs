use crate::config::Config;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

#[derive(Default)]
pub struct ThemeSelectorComponent;

impl ThemeSelectorComponent {
    pub fn new() -> Self {
        Self
    }

    /// Convert AnsiColor to ratatui Color
    fn convert_color_ansi(color: &crate::config::types::AnsiColor) -> Color {
        match color {
            crate::config::types::AnsiColor::Color16 { c16 } => match *c16 {
                0 => Color::Black,
                1 => Color::Red,
                2 => Color::Green,
                3 => Color::Yellow,
                4 => Color::Blue,
                5 => Color::Magenta,
                6 => Color::Cyan,
                7 => Color::White,
                8 => Color::DarkGray,
                9 => Color::LightRed,
                10 => Color::LightGreen,
                11 => Color::LightYellow,
                12 => Color::LightBlue,
                13 => Color::LightMagenta,
                14 => Color::LightCyan,
                15 => Color::White,
                _ => Color::Gray,
            },
            crate::config::types::AnsiColor::Color256 { c256 } => Color::Indexed(*c256),
            crate::config::types::AnsiColor::Rgb { r, g, b } => Color::Rgb(*r, *g, *b),
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, config: &Config) {
        let is_modified = config.is_modified_from_theme();
        let modified_indicator = if is_modified { "*" } else { "" };

        // Get all available themes dynamically and filter out empty names
        let available_themes: Vec<String> = crate::ui::themes::ThemePresets::list_available_themes()
            .into_iter()
            .filter(|theme| !theme.trim().is_empty())
            .collect();

        // Calculate available width (minus borders and spacing)
        let content_width = area.width.saturating_sub(2); // Remove borders

        // Build theme options with auto-wrapping
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut first_line = true;

        for (i, theme) in available_themes.iter().enumerate() {
            let marker = if config.theme == theme.as_str() {
                "[✓]"
            } else {
                "[ ]"
            };
            let theme_part = format!("{} {}", marker, theme);
            let separator = if i == 0 { "" } else { "  " };
            let part_with_sep = format!("{}{}", separator, theme_part);

            // Check if this part fits in current line using unicode width
            let would_fit = current_line.width() + part_with_sep.width() <= content_width as usize;

            if would_fit || first_line {
                current_line.push_str(&part_with_sep);
                first_line = false;
            } else {
                // Start new line
                lines.push(current_line);
                current_line = theme_part; // No indent for continuation lines
            }
        }

        if !current_line.trim().is_empty() {
            lines.push(current_line);
        }

        // Build text lines
        let mut text_lines = Vec::new();

        // Add theme selection lines
        for line in lines {
            text_lines.push(Line::from(line));
        }

        // Add empty line as separator
        text_lines.push(Line::from(""));

        // Add color preview line
        let mut preview_spans = Vec::new();

        // Add label
        preview_spans.push(Span::raw("预览: "));

        // Helper function to find segment by ID and get its background color
        let get_bg_color = |segment_id: crate::config::types::SegmentId| -> Color {
            config.segments
                .iter()
                .find(|s| s.id == segment_id)
                .and_then(|s| s.colors.background.as_ref())
                .map(|c| Self::convert_color_ansi(c))
                .unwrap_or(Color::Gray)
        };

        // Add sample segments with different colors
        let segments = [
            (" 目录 ", crate::config::types::SegmentId::Directory),
            (" 分支 ", crate::config::types::SegmentId::Git),
            (" 模型 ", crate::config::types::SegmentId::Model),
            (" 上下文 ", crate::config::types::SegmentId::ContextWindow),
        ];

        for (text, segment_id) in segments {
            let bg_color = get_bg_color(segment_id);
            let fg_color = Color::Rgb(255, 255, 255); // White text for contrast

            preview_spans.push(Span::styled(
                text,
                Style::default().fg(fg_color).bg(bg_color),
            ));
            preview_spans.push(Span::raw(" "));
        }

        text_lines.push(Line::from(preview_spans));

        // Add separator display line
        let separator_line = format!("分隔符: \"{}\"", config.style.separator);
        text_lines.push(Line::from(separator_line));

        let title = format!("主题: {}{}", config.theme, modified_indicator);
        let theme_selector = Paragraph::new(text_lines)
            .block(Block::default().borders(Borders::ALL).title(title))
            .wrap(ratatui::widgets::Wrap { trim: false });
        f.render_widget(theme_selector, area);
    }
}
