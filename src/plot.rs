use anyhow::Result;
use chrono::{DateTime, Utc};
use image::{ImageBuffer, Rgb};
use plotters::backend::BitMapBackend;
use plotters::prelude::*;
use terminal_size::{terminal_size, Width};
use textplots::{
    AxisBuilder, Chart, LabelBuilder, LabelFormat, Plot, Shape, TickDisplay, TickDisplayBuilder,
};

pub fn plot_ascii(
    buckets: &[(DateTime<Utc>, usize)],
    time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    bucket_size_seconds: f64,
    pattern: &str,
    _files: &[String],
    y_zero: bool,
) -> Result<()> {
    if buckets.is_empty() {
        println!("No data to plot.");
        return Ok(());
    }
    // Use actual time range if provided, otherwise fall back to bucket range
    let (first_ts, last_ts, time_range_seconds) = if let Some((first, last)) = time_range {
        let range_seconds = (last.timestamp() - first.timestamp()) as f32;
        (first, last, range_seconds)
    } else {
        let (first, _) = buckets.first().unwrap();
        let (last, _) = buckets.last().unwrap();
        let range_seconds = (last.timestamp() - first.timestamp()) as f32;
        (*first, *last, range_seconds)
    };

    // Calculate dimensions
    let max_count = buckets.iter().map(|(_, c)| c).max().unwrap_or(&0);
    let duration_hours = time_range_seconds / 3600.0;

    // Format files list
    let files_str = if _files.is_empty() {
        "stdin".to_string()
    } else if _files.len() == 1 {
        _files[0].clone()
    } else {
        format!("{} files", _files.len())
    };

    // Convert to points for textplots using bucket indices
    let points: Vec<(f32, f32)> = buckets
        .iter()
        .enumerate()
        .map(|(i, (_, count))| (i as f32, *count as f32))
        .collect();

    let x_min = 0.0;
    let x_max = (points.len().saturating_sub(1)) as f32;

    // Calculate Y-axis range
    let y_min = if y_zero {
        0.0
    } else {
        let min_count = buckets.iter().map(|(_, c)| *c).min().unwrap_or(0) as f32;
        (min_count * 0.9).max(0.0) // 10% padding below min, but not negative
    };
    let y_max = *max_count as f32;

    // Build and display chart
    render_chart(
        &points,
        x_min,
        x_max,
        y_min,
        y_max,
        pattern,
        bucket_size_seconds,
        &files_str,
    );

    let bucket_count = buckets.len();

    // Enhanced x-axis information
    if bucket_size_seconds < 1.0 {
        println!(
            "X-axis: Time offset (0-{:.1}s) | Buckets: {} ({:.1}s each)",
            time_range_seconds, bucket_count, bucket_size_seconds
        );
    } else if bucket_size_seconds < 60.0 {
        println!(
            "X-axis: Time offset (0-{:.0}s) | Buckets: {} ({:.0}s each)",
            time_range_seconds, bucket_count, bucket_size_seconds
        );
    } else if bucket_size_seconds < 3600.0 {
        println!(
            "X-axis: Time offset (0-{:.0}s) | Buckets: {} ({:.0}m each)",
            time_range_seconds,
            bucket_count,
            bucket_size_seconds / 60.0
        );
    } else {
        println!(
            "X-axis: Time offset (0-{:.1}h) | Buckets: {} ({:.1}h each)",
            duration_hours,
            bucket_count,
            bucket_size_seconds / 3600.0
        );
    }

    println!("Y-axis: Match count (max: {})", max_count);

    // Show time range
    println!(
        "Time range: {} to {}",
        first_ts.format("%Y-%m-%d %H:%M:%S"),
        last_ts.format("%Y-%m-%d %H:%M:%S")
    );

    Ok(())
}

fn chart_width_for_terminal() -> u32 {
    if let Some((Width(w), _)) = terminal_size() {
        // textplots internally halves width, so compensate
        ((w as u32 * 2) as f32 * 0.87).round() as u32
    } else {
        160 // fallback (80 * 2)
    }
}

#[allow(clippy::too_many_arguments)]
fn render_chart(
    points: &[(f32, f32)],
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
    pattern: &str,
    _bucket_size_seconds: f64,
    files_str: &str,
) {
    let term = console::Term::stdout();

    // calculate proper width
    let chart_width = chart_width_for_terminal();
    let _line_width = (chart_width / 2) - 10;
    let chart_height = (((chart_width / 2) as f32) / ((1.0 / 0.635) as f32)).round() as u32;

    term.hide_cursor().unwrap();
    term.clear_screen().unwrap();
    term.move_cursor_to(0, 0).unwrap();

    println!("\nPattern: \"{}\" | Files: {}\n", pattern, files_str);

    Chart::new_with_y_range(chart_width, chart_height, x_min, x_max, y_min, y_max)
        .lineplot(&Shape::Lines(points))
        .x_axis_style(textplots::LineStyle::Solid)
        .y_axis_style(textplots::LineStyle::Solid)
        .y_tick_display(TickDisplay::Sparse)
        .x_label_format(LabelFormat::Value)
        .y_label_format(LabelFormat::Value)
        .nice();

    term.show_cursor().unwrap();
}

pub fn plot_png(buckets: &[(DateTime<Utc>, usize)], output_file: &str) -> Result<()> {
    if buckets.is_empty() {
        anyhow::bail!("No data to plot.");
    }

    // Create a buffer for the bitmap
    const WIDTH: u32 = 1200;
    const HEIGHT: u32 = 600;
    let mut buffer = vec![0u8; (WIDTH * HEIGHT * 3) as usize];

    {
        let root = BitMapBackend::with_buffer(&mut buffer, (WIDTH, HEIGHT)).into_drawing_area();
        root.fill(&WHITE)?;

        let max_count = buckets.iter().map(|(_, c)| c).max().unwrap_or(&0);
        let (first_ts, _) = buckets.first().unwrap();
        let (last_ts, _) = buckets.last().unwrap();

        let mut chart = ChartBuilder::on(&root)
            .caption("Log Matches Over Time", ("sans-serif", 50).into_font())
            .margin(10)
            .x_label_area_size(50)
            .y_label_area_size(60)
            .build_cartesian_2d(
                first_ts.timestamp()..last_ts.timestamp(),
                0..*max_count + (max_count / 10).max(1),
            )?;

        chart
            .configure_mesh()
            .x_desc("Time")
            .y_desc("Count")
            .x_label_formatter(&|x| {
                DateTime::from_timestamp(*x, 0)
                    .map(|dt| dt.format("%H:%M").to_string())
                    .unwrap_or_default()
            })
            .axis_desc_style(("sans-serif", 20))
            .label_style(("sans-serif", 15))
            .draw()?;

        // Draw line chart
        chart
            .draw_series(LineSeries::new(
                buckets.iter().map(|(ts, count)| (ts.timestamp(), *count)),
                &BLUE.mix(0.8),
            ))?
            .label("Matches")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE));

        // Draw points
        chart.draw_series(
            buckets
                .iter()
                .map(|(ts, count)| Circle::new((ts.timestamp(), *count), 4, BLUE.filled())),
        )?;

        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.8))
            .border_style(BLACK)
            .draw()?;

        root.present()?;
    }

    // Convert buffer to ImageBuffer and save as PNG
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(WIDTH, HEIGHT, buffer)
        .ok_or_else(|| anyhow::anyhow!("Failed to create image from buffer"))?;

    img.save(output_file)?;

    println!("Chart saved to: {}", output_file);

    Ok(())
}
