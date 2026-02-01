use crate::providers::{ClaudeData, CodexData, CopilotData, GeminiData, ProviderData, ProviderStatus};
use tabled::{builder::Builder, settings::Style, settings::Color, settings::span::Span, settings::style::HorizontalLine, settings::themes::BorderCorrection};

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

    // Build horizontal lines: double line after header + dotted lines between providers
    let double_line = HorizontalLine::full('═', '╪', '╞', '╡');
    let dotted_line = HorizontalLine::full('┄', '┼', '├', '┤');

    // Collect separator positions (after each provider except the last)
    let mut separator_rows: Vec<usize> = Vec::new();
    for (i, (start_row, row_count)) in provider_spans.iter().enumerate() {
        if i < provider_spans.len() - 1 {
            separator_rows.push(start_row + row_count);
        }
    }

    // Apply style based on number of separators needed
    match separator_rows.len() {
        0 => {
            let style = Style::rounded().horizontals([(1, double_line)]);
            table.with(style);
        }
        1 => {
            let style = Style::rounded().horizontals([
                (1, double_line),
                (separator_rows[0], dotted_line),
            ]);
            table.with(style);
        }
        2 => {
            let style = Style::rounded().horizontals([
                (1, double_line),
                (separator_rows[0], dotted_line.clone()),
                (separator_rows[1], dotted_line),
            ]);
            table.with(style);
        }
        3 => {
            let style = Style::rounded().horizontals([
                (1, double_line),
                (separator_rows[0], dotted_line.clone()),
                (separator_rows[1], dotted_line.clone()),
                (separator_rows[2], dotted_line),
            ]);
            table.with(style);
        }
        _ => {
            // Fallback for 4+ separators (5+ providers)
            let style = Style::rounded().horizontals([(1, double_line)]);
            table.with(style);
        }
    }
    
    // Apply cell spanning for provider column only (status is now per-row)
    for (start_row, row_count) in &provider_spans {
        if *row_count > 1 {
            table.modify((*start_row, 0), Span::row(*row_count as isize));
        }
    }
    
    // Apply colors to cells (using tabled's Color, not ANSI codes)
    if !no_color {
        use tabled::settings::object::Rows;

        // Bold header row
        table.modify(Rows::first(), Color::BOLD);

        // Color the Provider column (column 0) in light blue for data rows only
        for (start_row, _) in &provider_spans {
            table.modify((*start_row, 0), Color::FG_BRIGHT_BLUE);
        }

        // Apply cell-specific colors (usage and status columns)
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
    _provider_data: &ProviderData,
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

    // Add one row per model with per-model status
    // Invert usage to show % USED (like other providers) instead of % remaining
    // 100% remaining becomes 0% used, 0% remaining becomes 100% used
    for (i, model) in data.models.iter().enumerate() {
        let reset_str = model.reset_time
            .map(|t| format_reset_time(t))
            .unwrap_or_else(|| "-".to_string());

        let used_percent = (100.0 - model.remaining_percent) as i32;
        let usage_str = format!("{}%", used_percent);
        let current_row = start_row + i;

        // Per-model status
        let row_status = get_row_status(used_percent);
        let status_text = format_status(row_status);

        if i == 0 {
            builder.push_record([
                provider_cell.clone(),
                model.model.clone(),
                usage_str,
                reset_str,
                status_text,
            ]);
        } else {
            builder.push_record([
                String::new(),
                model.model.clone(),
                usage_str,
                reset_str,
                status_text,
            ]);
        }

        // Track colors for usage (column 2) and status (column 4)
        if !no_color {
            let usage_color = get_usage_color(used_percent);
            let status_color = get_status_color(row_status);
            cell_colors.push((current_row, 2, usage_color));
            cell_colors.push((current_row, 4, status_color));
        }
    }

    // If no models, add a row indicating that
    if data.models.is_empty() {
        builder.push_record([
            provider_cell,
            "-".to_string(),
            "-".to_string(),
            "-".to_string(),
            "✓ OK".to_string(),
        ]);
        if !no_color {
            cell_colors.push((start_row, 4, Color::FG_GREEN));
        }
    }
}

fn add_codex_rows(
    builder: &mut Builder,
    data: &CodexData,
    _provider_data: &ProviderData,
    no_color: bool,
    start_row: usize,
    cell_colors: &mut Vec<(usize, usize, Color)>
) {
    let name = "Codex".to_string();

    // Primary window with per-window status
    let primary_percent = data.primary_window.used_percent;
    let primary_usage = format!("{}%", primary_percent);
    let primary_reset = format_seconds(data.primary_window.resets_in_seconds);
    let primary_status = get_row_status(primary_percent);

    builder.push_record([
        name.clone(),
        "Primary".to_string(),
        primary_usage,
        primary_reset,
        format_status(primary_status),
    ]);

    if !no_color {
        cell_colors.push((start_row, 2, get_usage_color(primary_percent)));
        cell_colors.push((start_row, 4, get_status_color(primary_status)));
    }

    // Secondary window with per-window status
    let secondary_percent = data.secondary_window.used_percent;
    let secondary_usage = format!("{}%", secondary_percent);
    let secondary_reset = format_seconds(data.secondary_window.resets_in_seconds);
    let secondary_status = get_row_status(secondary_percent);

    builder.push_record([
        String::new(),
        "Secondary".to_string(),
        secondary_usage,
        secondary_reset,
        format_status(secondary_status),
    ]);

    if !no_color {
        cell_colors.push((start_row + 1, 2, get_usage_color(secondary_percent)));
        cell_colors.push((start_row + 1, 4, get_status_color(secondary_status)));
    }
}

fn add_copilot_rows(
    builder: &mut Builder,
    data: &CopilotData,
    _provider_data: &ProviderData,
    no_color: bool,
    start_row: usize,
    cell_colors: &mut Vec<(usize, usize, Color)>
) {
    let name = "Copilot".to_string();

    // Calculate usage percentage (inverted from remaining to align with other providers)
    // 100% remaining becomes 0% used, 0% remaining becomes 100% used
    let used_percent = if data.premium_entitlement > 0 {
        let remaining_fraction = data.premium_remaining as f64 / data.premium_entitlement as f64;
        ((1.0 - remaining_fraction) * 100.0).clamp(0.0, 100.0) as i32
    } else {
        0
    };

    let usage_str = format!("{}%", used_percent);
    let row_status = get_row_status(used_percent);

    builder.push_record([
        name,
        "Premium Requests".to_string(),
        usage_str,
        data.quota_reset_date.clone(),
        format_status(row_status),
    ]);

    if !no_color {
        cell_colors.push((start_row, 2, get_usage_color(used_percent)));
        cell_colors.push((start_row, 4, get_status_color(row_status)));
    }
}

fn add_claude_rows(
    builder: &mut Builder,
    data: &ClaudeData,
    _provider_data: &ProviderData,
    no_color: bool,
    start_row: usize,
    cell_colors: &mut Vec<(usize, usize, Color)>
) {
    let name = "Claude".to_string();

    // 5-hour window with per-window status
    let five_h_percent = data.five_hour.utilization as i32;
    let five_h_usage = format!("{}%", five_h_percent);
    let five_h_reset = data.five_hour.resets_at
        .map(|t| format_reset_time(t))
        .unwrap_or_else(|| "-".to_string());
    let five_h_status = get_row_status(five_h_percent);

    builder.push_record([
        name.clone(),
        "5h Window".to_string(),
        five_h_usage,
        five_h_reset,
        format_status(five_h_status),
    ]);

    if !no_color {
        cell_colors.push((start_row, 2, get_usage_color(five_h_percent)));
        cell_colors.push((start_row, 4, get_status_color(five_h_status)));
    }

    // 7-day window with per-window status
    let seven_d_percent = data.seven_day.utilization as i32;
    let seven_d_usage = format!("{}%", seven_d_percent);
    let seven_d_reset = data.seven_day.resets_at
        .map(|t| format_reset_time(t))
        .unwrap_or_else(|| "-".to_string());
    let seven_d_status = get_row_status(seven_d_percent);

    builder.push_record([
        String::new(),
        "7d Window".to_string(),
        seven_d_usage,
        seven_d_reset,
        format_status(seven_d_status),
    ]);

    if !no_color {
        cell_colors.push((start_row + 1, 2, get_usage_color(seven_d_percent)));
        cell_colors.push((start_row + 1, 4, get_status_color(seven_d_status)));
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

/// Get status based on usage percentage (for per-row status)
/// Lower usage = OK, higher = warning
fn get_row_status(used_percent: i32) -> ProviderStatus {
    if used_percent >= 80 {
        ProviderStatus::Warning
    } else {
        ProviderStatus::Ok
    }
}
