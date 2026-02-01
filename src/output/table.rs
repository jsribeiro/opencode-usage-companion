use crate::providers::{ClaudeData, CodexData, CopilotData, GeminiData, ProviderData, ProviderStatus};
use tabled::{builder::Builder, settings::Style, settings::Color, settings::span::Span, settings::style::HorizontalLine, settings::themes::BorderCorrection};
use colored::Colorize;

/// Format data as a pretty table with UTF-8 borders
/// Features:
/// - Solid UTF-8 lines
/// - Double line after header
/// - Cell spanning for provider and status columns
/// - Proper colorization using tabled's Color settings (no ANSI codes in content)
pub fn format_table(data: &[ProviderData], no_color: bool) -> String {
    if data.is_empty() {
        return "No provider data available.".to_string();
    }

    let mut builder = Builder::default();
    
    // Add header as first record
    builder.push_record(["Provider", "Model", "Usage", "Resets", "Status"]);

    // Track row indices and colors for each provider
    let mut provider_spans: Vec<(usize, usize)> = Vec::new(); // (start_row, row_count)
    let mut cell_colors: Vec<(usize, usize, Color)> = Vec::new(); // (row, col, color)
    let mut current_row = 1usize; // Start after header

    for provider_data in data {
        let row_count = count_provider_rows(provider_data);
        if row_count > 0 {
            provider_spans.push((current_row, row_count));
        }
        add_provider_rows(&mut builder, provider_data, no_color, current_row, &mut cell_colors);
        current_row += row_count;
    }

    let mut table = builder.build();
    
    // Apply rounded UTF-8 style with a double horizontal line after header
    let double_line = HorizontalLine::full('═', '┼', '├', '┤');
    let style = Style::rounded()
        .horizontals([(1, double_line)]);
    
    table.with(style);
    
    // Apply cell spanning for providers with multiple rows
    for (start_row, row_count) in provider_spans {
        if row_count > 1 {
            table.modify((start_row, 0), Span::row(row_count as isize));
            table.modify((start_row, 4), Span::row(row_count as isize));
        }
    }
    
    // Apply colors to cells (using tabled's Color, not ANSI codes)
    if !no_color {
        // Color header row with background
        use tabled::settings::object::Rows;
        table.modify(Rows::first(), Color::BG_BLUE | Color::FG_WHITE);
        
        // Apply cell-specific colors
        for (row, col, color) in cell_colors {
            table.modify((row, col), color);
        }
    }
    
    // Correct borders for spanned cells
    table.with(BorderCorrection::span());

    table.to_string()
}

fn count_provider_rows(data: &ProviderData) -> usize {
    match data {
        ProviderData::Gemini(gemini) => {
            if gemini.models.is_empty() { 1 } else { gemini.models.len() }
        }
        ProviderData::Codex(_) => 2,
        ProviderData::Copilot(_) => 1,
        ProviderData::Claude(_) => 2,
    }
}

fn add_provider_rows(
    builder: &mut Builder, 
    data: &ProviderData, 
    no_color: bool, 
    start_row: usize,
    cell_colors: &mut Vec<(usize, usize, Color)>
) {
    match data {
        ProviderData::Gemini(gemini) => add_gemini_rows(builder, gemini, data, no_color, start_row, cell_colors),
        ProviderData::Codex(codex) => add_codex_rows(builder, codex, data, no_color, start_row, cell_colors),
        ProviderData::Copilot(copilot) => add_copilot_rows(builder, copilot, data, no_color, start_row, cell_colors),
        ProviderData::Claude(claude) => add_claude_rows(builder, claude, data, no_color, start_row, cell_colors),
    }
}

fn add_gemini_rows(
    builder: &mut Builder, 
    data: &GeminiData, 
    provider_data: &ProviderData, 
    no_color: bool,
    start_row: usize,
    cell_colors: &mut Vec<(usize, usize, Color)>
) {
    let provider_name = if data.is_active {
        "Gemini".to_string()
    } else {
        "Gemini [inactive]".to_string()
    };

    let provider_cell = format!("{}\n{}", provider_name, data.account_email);
    let status_text = format_status(provider_data.status());
    let status_color = get_status_color(provider_data.status());

    // Add one row per model
    // Invert usage to show % USED (like other providers) instead of % remaining
    // 100% remaining becomes 0% used, 0% remaining becomes 100% used
    for (i, model) in data.models.iter().enumerate() {
        let reset_str = model.reset_time
            .map(|t| format_reset_time(t))
            .unwrap_or_else(|| "-".to_string());
        
        let used_percent = 100.0 - model.remaining_percent;
        let usage_str = format!("{:.0}%", used_percent);
        let current_row = start_row + i;

        if i == 0 {
            builder.push_record([
                provider_cell.clone(),
                model.model.clone(),
                usage_str,
                reset_str,
                status_text.clone(),
            ]);
        } else {
            builder.push_record([
                String::new(),
                model.model.clone(),
                usage_str,
                reset_str,
                String::new(),
            ]);
        }

        // Track colors for usage column (column 2) - now using % used (inverted)
        // High % used = red (bad), Low % used = green (good)
        if !no_color {
            let usage_color = get_usage_color(used_percent as i32);
            cell_colors.push((current_row, 2, usage_color));
            // Only color the status on the first row of each provider
            if i == 0 {
                cell_colors.push((current_row, 4, status_color.clone()));
            }
        }
    }

    // If no models, add a row indicating that
    if data.models.is_empty() {
        builder.push_record([
            provider_cell,
            "-".to_string(),
            "-".to_string(),
            "-".to_string(),
            status_text,
        ]);
        if !no_color {
            cell_colors.push((start_row, 4, status_color));
        }
    }
}

fn add_codex_rows(
    builder: &mut Builder, 
    data: &CodexData, 
    provider_data: &ProviderData, 
    no_color: bool,
    start_row: usize,
    cell_colors: &mut Vec<(usize, usize, Color)>
) {
    let name = "Codex".to_string();
    let status_text = format_status(provider_data.status());
    let status_color = get_status_color(provider_data.status());

    // Primary window (usage is "used" percentage, so lower is better)
    let primary_usage = format!("{}%", data.primary_window.used_percent);
    let primary_reset = format_seconds(data.primary_window.resets_in_seconds);
    
    builder.push_record([
        name.clone(),
        "Primary".to_string(),
        primary_usage.clone(),
        primary_reset,
        status_text.clone(),
    ]);
    
    // Track color for primary usage (column 2) - for Codex, lower usage is better (green)
    if !no_color {
        let color = get_usage_color(data.primary_window.used_percent as i32);
        cell_colors.push((start_row, 2, color));
        cell_colors.push((start_row, 4, status_color));
    }
    
    // Secondary window
    let secondary_usage = format!("{}%", data.secondary_window.used_percent);
    let secondary_reset = format_seconds(data.secondary_window.resets_in_seconds);
    
    builder.push_record([
        String::new(),
        "Secondary".to_string(),
        secondary_usage.clone(),
        secondary_reset,
        String::new(),
    ]);

    // Track color for secondary usage
    if !no_color {
        let color = get_usage_color(data.secondary_window.used_percent as i32);
        cell_colors.push((start_row + 1, 2, color));
    }
}

fn add_copilot_rows(
    builder: &mut Builder, 
    data: &CopilotData, 
    provider_data: &ProviderData, 
    no_color: bool,
    start_row: usize,
    cell_colors: &mut Vec<(usize, usize, Color)>
) {
    let name = "Copilot".to_string();
    let status_text = format_status(provider_data.status());
    let status_color = get_status_color(provider_data.status());

    let remaining_percent = if data.premium_entitlement > 0 {
        ((data.premium_remaining as f64 / data.premium_entitlement as f64) * 100.0) as i32
    } else {
        0
    };
    
    let usage_percent = format!("{}%", remaining_percent);
    
    builder.push_record([
        name,
        "Premium Requests".to_string(),
        usage_percent.clone(),
        data.quota_reset_date.clone(),
        status_text,
    ]);

    // Track colors for usage (column 2) and status (column 4)
    if !no_color {
        let usage_color = get_percentage_color(remaining_percent as f64);
        cell_colors.push((start_row, 2, usage_color));
        cell_colors.push((start_row, 4, status_color));
    }
}

fn add_claude_rows(
    builder: &mut Builder, 
    data: &ClaudeData, 
    provider_data: &ProviderData, 
    no_color: bool,
    start_row: usize,
    cell_colors: &mut Vec<(usize, usize, Color)>
) {
    let name = "Claude".to_string();
    let status_text = format_status(provider_data.status());
    let status_color = get_status_color(provider_data.status());

    // 5-hour window (utilization is "used" percentage, so lower is better)
    let five_h_usage = format!("{:.0}%", data.five_hour.utilization);
    let five_h_reset = data.five_hour.resets_at
        .map(|t| format_reset_time(t))
        .unwrap_or_else(|| "-".to_string());
    
    builder.push_record([
        name.clone(),
        "5h Window".to_string(),
        five_h_usage.clone(),
        five_h_reset,
        status_text.clone(),
    ]);
    
    // Track colors for 5h window (column 2) and status (column 4)
    if !no_color {
        let usage_color = get_usage_color(data.five_hour.utilization as i32);
        cell_colors.push((start_row, 2, usage_color));
        cell_colors.push((start_row, 4, status_color));
    }
    
    // 7-day window
    let seven_d_usage = format!("{:.0}%", data.seven_day.utilization);
    let seven_d_reset = data.seven_day.resets_at
        .map(|t| format_reset_time(t))
        .unwrap_or_else(|| "-".to_string());
    
    builder.push_record([
        String::new(),
        "7d Window".to_string(),
        seven_d_usage.clone(),
        seven_d_reset,
        String::new(),
    ]);

    // Track color for 7d window (column 2)
    if !no_color {
        let color = get_usage_color(data.seven_day.utilization as i32);
        cell_colors.push((start_row + 1, 2, color));
    }
}

fn colorize_percentage(percent: f64, text: &str) -> String {
    if percent >= 80.0 {
        text.green().to_string()
    } else if percent >= 50.0 {
        text.yellow().to_string()
    } else if percent >= 20.0 {
        text.bright_yellow().to_string()
    } else {
        text.red().to_string()
    }
}

fn colorize_usage_percentage(percent: i32, text: &str) -> String {
    if percent < 50 {
        text.green().to_string()
    } else if percent < 80 {
        text.yellow().to_string()
    } else {
        text.red().to_string()
    }
}

fn format_reset_time(dt: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = dt.signed_duration_since(now);

    if duration.num_hours() > 24 {
        format!("{}d", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{}h {}m", duration.num_hours(), duration.num_minutes() % 60)
    } else {
        format!("{}m", duration.num_minutes())
    }
}

fn format_seconds(seconds: i64) -> String {
    if seconds > 86400 {
        format!("{}d", seconds / 86400)
    } else if seconds > 3600 {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    } else {
        format!("{}m", seconds / 60)
    }
}

fn format_status(status: ProviderStatus) -> String {
    // Return plain text with icons - colors are applied via tabled's Color settings
    match status {
        ProviderStatus::Ok => "✓ OK".to_string(),
        ProviderStatus::Warning => "⚠️ WARNING".to_string(),
        ProviderStatus::Error => "✗ ERROR".to_string(),
    }
}

/// Get tabled Color for status
fn get_status_color(status: ProviderStatus) -> Color {
    match status {
        ProviderStatus::Ok => Color::FG_GREEN,
        ProviderStatus::Warning => Color::FG_YELLOW,
        ProviderStatus::Error => Color::FG_RED,
    }
}

/// Get tabled Color for percentage values (for remaining/quota percentages)
/// Higher percentage = better (green), lower = warning (yellow/red)
fn get_percentage_color(percent: f64) -> Color {
    if percent >= 80.0 {
        Color::FG_GREEN
    } else if percent >= 50.0 {
        Color::FG_YELLOW
    } else if percent >= 20.0 {
        Color::FG_BRIGHT_YELLOW
    } else {
        Color::FG_RED
    }
}

/// Get tabled Color for usage percentages (for utilization/used percentages)
/// Lower usage = better (green), higher = warning (yellow/red)
fn get_usage_color(percent: i32) -> Color {
    if percent < 50 {
        Color::FG_GREEN
    } else if percent < 80 {
        Color::FG_YELLOW
    } else {
        Color::FG_RED
    }
}
