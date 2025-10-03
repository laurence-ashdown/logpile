use anyhow::Result;
use chrono::{DateTime, Utc};
use image::{ImageBuffer, Rgb};
use plotters::backend::BitMapBackend;
use plotters::prelude::*;
use textplots::{Chart, Plot, Shape};

pub fn plot_ascii(buckets: &[(DateTime<Utc>, usize)]) -> Result<()> {
    if buckets.is_empty() {
        println!("No data to plot.");
        return Ok(());
    }

    // Convert to points for textplots
    let points: Vec<(f32, f32)> = buckets
        .iter()
        .enumerate()
        .map(|(i, (_, count))| (i as f32, *count as f32))
        .collect();

    // Calculate dimensions
    let max_count = buckets.iter().map(|(_, c)| c).max().unwrap_or(&0);

    println!("\nLog Matches Over Time");
    println!("{}", "=".repeat(60));

    Chart::new(180, 60, 0.0, points.len() as f32)
        .lineplot(&Shape::Lines(&points))
        .display();

    println!("\n{}", "=".repeat(60));
    println!("X-axis: Bucket index (0-{})", buckets.len() - 1);
    println!("Y-axis: Match count (max: {})", max_count);

    // Show time range
    if let (Some((first_ts, _)), Some((last_ts, _))) = (buckets.first(), buckets.last()) {
        println!(
            "Time range: {} to {}",
            first_ts.format("%Y-%m-%d %H:%M:%S"),
            last_ts.format("%Y-%m-%d %H:%M:%S")
        );
    }

    Ok(())
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
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

        // Draw points
        chart.draw_series(
            buckets
                .iter()
                .map(|(ts, count)| Circle::new((ts.timestamp(), *count), 4, BLUE.filled())),
        )?;

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
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
