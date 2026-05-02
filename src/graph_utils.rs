use plotters::prelude::*;
use std::path::Path;

/// Plot multiple (x,y) series on the same line chart and save as an image.
///
/// `path` : output file path (png recommended)
/// `title`, `x_label`, `y_label` : chart labels
/// `series` : slice of tuples (series_name, Vec<(x,y)>)
pub fn plot_multi_series_line<P: AsRef<Path>>(
    path: P,
    title: &str,
    x_label: &str,
    y_label: &str,
    series: &[(&str, Vec<(f64, f64)>)],
) -> Result<(), Box<dyn std::error::Error>> {
    let path = path.as_ref();
    let root = BitMapBackend::new(path, (1280, 720)).into_drawing_area();
    root.fill(&WHITE)?;

    // collect ranges
    let mut xs: Vec<f64> = Vec::new();
    let mut ys: Vec<f64> = Vec::new();
    for (_, s) in series.iter() {
        for (x, y) in s.iter() {
            xs.push(*x);
            ys.push(*y);
        }
    }

    if xs.is_empty() || ys.is_empty() {
        return Err("no data provided".into());
    }

    let x_min = xs.iter().cloned().fold(f64::INFINITY, f64::min);
    let x_max = xs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let y_min = ys.iter().cloned().fold(f64::INFINITY, f64::min);
    let y_max = ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 30))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

    chart
        .configure_mesh()
        .x_desc(x_label)
        .y_desc(y_label)
        .draw()?;

    let colors = [&RED, &BLUE, &GREEN, &CYAN, &MAGENTA, &YELLOW, &BLACK];

    for (i, (name, s)) in series.iter().enumerate() {
        let color = colors[i % colors.len()];
        let line = s.iter().map(|(x, y)| (*x, *y));
        chart
            .draw_series(LineSeries::new(line, color.stroke_width(2)))?
            .label(*name)
            .legend(move |(x, y)| {
                PathElement::new(vec![(x, y), (x + 20, y)], color.stroke_width(2))
            });
    }

    chart
        .configure_series_labels()
        .border_style(&BLACK)
        .draw()?;

    Ok(())
}

/// Convenience: plot Y values against their indices for multiple series.
pub fn plot_series_y_vs_index<P: AsRef<Path>>(
    path: P,
    title: &str,
    y_label: &str,
    series: &[(&str, Vec<f64>)],
) -> Result<(), Box<dyn std::error::Error>> {
    // convert to (index, value)
    let mut converted: Vec<(&str, Vec<(f64, f64)>)> = Vec::new();
    let mut max_x = 0usize;
    for (name, s) in series.iter() {
        let mut vec_xy: Vec<(f64, f64)> = Vec::with_capacity(s.len());
        for (i, v) in s.iter().enumerate() {
            vec_xy.push((i as f64, *v));
        }
        max_x = max_x.max(s.len());
        converted.push((*name, vec_xy));
    }

    if max_x == 0 {
        return Err("no data provided".into());
    }

    // determine x/y ranges
    let mut ys: Vec<f64> = Vec::new();
    for (_, s) in converted.iter() {
        for (_, y) in s.iter() {
            ys.push(*y);
        }
    }
    let y_min = ys.iter().cloned().fold(f64::INFINITY, f64::min);
    let y_max = ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    plot_multi_series_line(
        path,
        title,
        "index",
        y_label,
        &converted
            .iter()
            .map(|(n, s)| (*n, s.clone()))
            .collect::<Vec<_>>(),
    )
}
