//! Plot the temperature figure and accepts signals that send temperature data points.

use anyhow::{Result, anyhow, bail};
use plotters::prelude::*;
use slint::{ComponentHandle, Weak};
use tokio::{sync::mpsc, time::Instant};

use crate::{
    app::{AppWindow, Logic},
    logger::{LogMessage, send_log_message_now},
    plots::{Measurements, PLOT_STYLE, PlotSizePx, TIME_INTERVAL_CLEANUP, TIME_RANGE_TO_KEEP, TemperatureDataPoint},
};

pub enum TemperaturePlotCommands {
    AddDataPoint(TemperatureDataPoint),
    SetUi(Weak<AppWindow>),
}

pub struct TemperaturePlot {
    measurements: Measurements,
    ui: Option<Weak<AppWindow>>,
    plot_size: PlotSizePx,
}

impl TemperaturePlot {
    /// Create a new and empty temperature plot.
    pub fn new(plot_size: PlotSizePx) -> Self {
        let measurements = Measurements::new_temperature();

        Self {
            measurements,
            ui: None,
            plot_size,
        }
    }

    /// Set the UI to this struct.
    pub fn set_ui(&mut self, ui: Weak<AppWindow>) {
        self.ui = Some(ui);
    }

    /// Make the plot for the temperature display with a logarithmic scenario and set it to the UI.
    fn plot_it(&self) -> Result<()> {
        let ui = match &self.ui {
            Some(ui) => ui,
            None => bail!("Cannot make temperature plot: UI not set."),
        };

        let mut pixel_buffer =
            slint::SharedPixelBuffer::new(self.plot_size.width, self.plot_size.height);
        let size = (pixel_buffer.width(), pixel_buffer.height());

        let backend = BitMapBackend::with_buffer(pixel_buffer.make_mut_bytes(), size);
        let root = backend.into_drawing_area();
        root.fill(&PLOT_STYLE.bg_color)?;

        let view = self.measurements.last_timerange_view(TIME_RANGE_TO_KEEP);

        // Bounds for the view we want
        let (min_ts, max_ts) = (
            view.timestamps()
                .next()
                .ok_or_else(|| anyhow!("Not enough data yet to plot."))?,
            view.timestamps()
                .last()
                .ok_or_else(|| anyhow!("Not enough data yet to plot."))?,
        );
        let (min_y, max_y) = (70.0, 300.0);

        let mut chart = ChartBuilder::on(&root)
            .margin(10)
            .set_label_area_size(LabelAreaPosition::Left, 90)
            .set_label_area_size(LabelAreaPosition::Bottom, 60)
            .build_cartesian_2d(min_ts..max_ts, min_y..max_y)?;

        let xlbl = "Time";
        let ylbl = "Temperature (K)";

        chart
            .configure_mesh()
            .light_line_style(PLOT_STYLE.mesh_minor_color)
            .bold_line_style(PLOT_STYLE.mesh_major_color)
            .label_style(
                (PLOT_STYLE.font, 24)
                    .into_font()
                    .color(&PLOT_STYLE.fg_color),
            )
            .axis_style(PLOT_STYLE.fg_color)
            .x_desc(xlbl)
            .x_labels(6)
            .x_label_formatter(&|xval| xval.format("%H:%M").to_string())
            .max_light_lines(4)
            .y_desc(ylbl)
            .y_labels(3)
            .y_label_formatter(&|yval| format!("{:.0}", yval))
            .draw()?;

        // Draw the first series
        chart.draw_series(LineSeries::new(
            view.iter_series_1(),
            &PLOT_STYLE.sample_color,
        ))?;

        // Draw the second series
        chart.draw_series(LineSeries::new(
            view.iter_series_2(),
            &PLOT_STYLE.bridge_color,
        ))?;

        // Draw the third series
        chart.draw_series(LineSeries::new(
            view.iter_series_3(),
            &PLOT_STYLE.cooler_color,
        ))?;

        // Avoid IO failure being ignored silently by manually calling present function
        root.present()?;

        drop(chart);
        drop(root);

        ui.upgrade_in_event_loop(move |ui| {
            let img = slint::Image::from_rgb8(pixel_buffer);
            ui.global::<Logic>().set_temperature_plot(img);
        })
        .expect("UI still exists");

        Ok(())
    }

    /// Make the plot and set it to the UI.
    ///
    /// TODO: Analyze what here actually needs to be done everytime and what can be done at init
    pub fn make_plot(&mut self) {
        if let Err(e) = self.plot_it() {
            send_log_message_now(LogMessage::new_error(&format!(
                "Failed to make temperature plot: {e}"
            )));
        }
    }
}

/// Pressure plot task: Receive temperature data points and update the plot and UI.
pub async fn temperature_plot_task(mut rx: mpsc::Receiver<TemperaturePlotCommands>) {
    let mut plot = TemperaturePlot::new(PlotSizePx {
        width: 800,
        height: 400,
    });

    let mut rx_shutdown = crate::HALT_SENDER.get().expect("Uninitialized").subscribe();

    let mut next_cleanup_time =  Instant::now() + TIME_INTERVAL_CLEANUP;

    loop {
        tokio::select! {
            Some(cmd) = rx.recv() => {
                match cmd {
                    TemperaturePlotCommands::AddDataPoint(dp) => {
                        // cleanup first?
                        if Instant::now() >= next_cleanup_time {
                            plot.measurements.retain(TIME_RANGE_TO_KEEP);
                            next_cleanup_time = Instant::now() + TIME_INTERVAL_CLEANUP;
                        }

                        plot.measurements.push_temperature(dp);
                        plot.make_plot();
                    }
                    TemperaturePlotCommands::SetUi(ui) => {
                        plot.set_ui(ui);
                    }
                }
            }
            _ = rx_shutdown.recv() => {
                    break;
            }
        }
    }
}

/// Get the command sender for the temperature plot.
fn get_temperature_plot_command_sender() -> mpsc::Sender<TemperaturePlotCommands> {
    crate::PLOT_TEMPERATURE_SENDER
        .get()
        .expect("Uninitialized")
        .clone()
}

/// Convenience function to await sending a temperature plot command.
///
/// If an error occurs, this error is logged. Otherwise, the program will continue
/// as normal.
pub async fn send_temperature_plot_cmd(cmd: TemperaturePlotCommands) {
    let sender = get_temperature_plot_command_sender();
    if let Err(e) = sender.send(cmd).await {
        send_log_message_now(LogMessage::new_error(&format!(
            "Failed to send temperature plot command: {}",
            e
        )));
    }
}

/// Convenience function to send a temperature plot command without awaiting.
///
/// If an error occurs, this error is logged. Otherwise, the program will continue
/// as normal.
pub fn send_temperature_plot_cmd_now(cmd: TemperaturePlotCommands) {
    let sender = get_temperature_plot_command_sender();
    if let Err(e) = sender.try_send(cmd) {
        send_log_message_now(LogMessage::new_error(&format!(
            "Failed to send temperature plot command now: {}",
            e
        )));
    }
}
