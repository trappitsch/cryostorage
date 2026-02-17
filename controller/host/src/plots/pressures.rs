//! Plot the pressure figure and accepts signals that send pressure data points.

use std::time::Duration;

use anyhow::{Result, anyhow, bail};
use plotters::prelude::*;
use slint::{ComponentHandle, Weak};
use tokio::sync::mpsc;

use crate::{
    app::{AppWindow, Logic}, logger::{LogMessage, send_log_message_now}, plots::{Measurements, PlotAttributes, PlotSizePx, PressureDataPoint, TIME_RANGE_TO_KEEP}
};

pub struct PressurePlot {
    measurements: Measurements,
    ui: Option<Weak<AppWindow>>,
    plot_size: PlotSizePx,
}

pub enum PressurePlotCommands {
    AddDataPoint(PressureDataPoint),
    SetUi(Weak<AppWindow>),
}

impl PressurePlot {
    /// Create a new and empty pressure plot.
    pub fn new(plot_size: PlotSizePx) -> Self {
        let attr_chamber = PlotAttributes {
            name: "Chamber".to_string(),
            color: RGBColor(137, 220, 235),
        };
        let attr_transfer = PlotAttributes {
            name: "Transfer".to_string(),
            color: RGBColor(249, 226, 175),
        };
        let measurements = Measurements::new_pressure(attr_chamber, attr_transfer);

        Self {
            measurements,
            ui: None,
            plot_size,
        }
    }

    /// Set the UI to this struct.
    pub fn set_ui(&mut self, ui: Weak<AppWindow>) {
        self.ui = Some(ui);
        println!("UI set for pressure plot");
    }

    /// Make the plot and set it to the UI.
    ///
    /// TODO: Analyze what here actually needs to be done everytime and what can be done at init
    pub fn make_plot(&mut self) -> Result<()> {
        let mut pixel_buffer =
            slint::SharedPixelBuffer::new(self.plot_size.width, self.plot_size.height);
        let size = (pixel_buffer.width(), pixel_buffer.height());

        let backend = BitMapBackend::with_buffer(pixel_buffer.make_mut_bytes(), size);
        let root = backend.into_drawing_area();

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
        let (min_y, max_y) = {
            let min_1 = view.series_1().fold(f64::INFINITY, f64::min);
            let max_1 = view.series_1().fold(f64::NEG_INFINITY, f64::max);
            let min_2 = view.series_2().fold(f64::INFINITY, f64::min);
            let max_2 = view.series_2().fold(f64::NEG_INFINITY, f64::max);
            (min_1.min(min_2), max_1.max(max_2))
        };

        // Build the cartesian chart
        let mut chart = ChartBuilder::on(&root)
            .caption("cpation", ("sans-serif", 28))
            .margin(10)
            .x_label_area_size(50)
            .y_label_area_size(60)
            .build_cartesian_2d(min_ts..max_ts, min_y..max_y)?;

        chart
            .configure_mesh()
            .x_desc("x desc")
            .y_desc("y desc")
            .draw()?;

        // Draw chamber pressure
        chart
            .draw_series(LineSeries::new(
                view.iter_pressure().map(|(ts, ch, _tr)| (ts, ch)),
                &RED,
            ))?
            .label(&self.measurements.attributes_1.name)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        // Draw transfer pressure
        chart
            .draw_series(LineSeries::new(
                view.iter_pressure().map(|(ts, _ch, tr)| (ts, tr)),
                &RED,
            ))?
            .label(&self.measurements.attributes_2.name)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

        // Legend
        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .label_font(("sans-serif", 20))
            .draw()?;

        // Avoid IO failure being ignored silently by manually calling present function
        root.present()?;

        drop(chart);
        drop(root);

        if let Some(ui) = &self.ui {
            ui.upgrade_in_event_loop(move |ui| {
                let img = slint::Image::from_rgb8(pixel_buffer);
                ui.global::<Logic>().set_vacuum_plot(img);
            })
            .expect("UI still exists");
        };

        Ok(())
    }
}

/// Pressure plot task: Receive pressure data points and update the plot and UI.
///
/// FIXME: Halt receiver
pub async fn pressure_plot_task(mut rx: mpsc::Receiver<PressurePlotCommands>) {

    println!("Starting pressure plot task");

    let mut plot = PressurePlot::new(PlotSizePx {
        width: 800,
        height: 600,
    });

    let mut rx_shutdown = crate::HALT_SENDER
        .get()
        .expect("Uninitialized")
        .subscribe();

    loop {
        tokio::select! {
            Some(cmd) = rx.recv() => {
                match cmd {
                    PressurePlotCommands::AddDataPoint(dp) => {
                        dbg!(&dp);
                        plot.measurements.push_pressure(dp);
                        match plot.make_plot() {
                            Ok(_) => {
                                println!("Updated plot");
                            },
                            Err(e) => {
                                println!("Error making plot: {e}")
                            }
                        }
                    }
                    PressurePlotCommands::SetUi(ui) => {
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

/// Get the command sender for the pressure plot.
fn get_pressur_plot_command_sender() -> mpsc::Sender<PressurePlotCommands> {
    crate::PLOT_PRESSURE_SENDER
        .get()
        .expect("Uninitialized")
        .clone()
}

/// Convenience function to await sending a pressure plot command.
/// 
/// If an error occurs, this error is logged. Otherwise, the program will continue
/// as normal.
pub async fn send_pressure_plot_cmd(cmd: PressurePlotCommands) {
    let sender = get_pressur_plot_command_sender();
    if let Err(e) = sender.send(cmd).await {
        send_log_message_now(LogMessage::new_error(&format!(
            "Failed to send pressure plot command: {}",
            e 
        )));
    }
}

/// Convenience function to send a pressure plot command without awaiting.
///
/// If an error occurs, this error is logged. Otherwise, the program will continue
/// as normal.
pub fn send_pressure_plot_cmd_now(cmd: PressurePlotCommands) {
    let sender = get_pressur_plot_command_sender();
    if let Err(e) = sender.try_send(cmd) {
        send_log_message_now(LogMessage::new_error(&format!(
            "Failed to send pressure plot command now: {}",
            e 
        )));
    }
}
