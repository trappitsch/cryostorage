//! Vacuum history - to save and to plot

use std::{env, io::Write, iter::zip, path::PathBuf};

use anyhow::Result;
use chrono::{DateTime, Local};
use measurements::Pressure;
use plotters::{chart::MeshStyle, prelude::*};
use slint::{ComponentHandle, Weak};

use crate::{
    CONFIG_FOLDER,
    app::{AppWindow, Logic},
};

const VACUUM_HISTORY: &str = "vacuum_history.csv";

/// Structure to hold the history data and deal with it.
pub struct VacuumHistory {
    /// Time stamps of the measurements
    timestamps: Vec<DateTime<Local>>,
    /// Pressure in the chamber.
    pressure_chamber_mbar: Vec<f64>,
    /// Pressure in the transfer.
    pressure_transfer_mbar: Vec<f64>,
    /// Plot size in pixels.
    plot_size: PlotSizePx,
    /// File name to store the history.
    fname: PathBuf,
    /// A weak reference to the UI.
    ui: Weak<AppWindow>,
    /// Plot style
    plot_style: PlotStyle,
}

impl VacuumHistory {
    /// Create a new VacuumHistory instance.
    pub fn new(plot_size: PlotSizePx, ui: Weak<AppWindow>) -> Self {
        let fname = env::home_dir()
            .expect("Home directory must be known")
            .join(CONFIG_FOLDER)
            .join(VACUUM_HISTORY);

        // If the file does not exist, create it with a header.
        if !fname.exists() {
            std::fs::create_dir_all(fname.parent().unwrap())
                .expect("Creating config folder must work");
            let mut file =
                std::fs::File::create(&fname).expect("Creating vacuum history file must work");
            writeln!(
                file,
                "Timestamp,Chamber_pressure_mbar,Transfer_pressure_mbar"
            )
            .expect("Writing header to vacuum history file must work");
        }

        let mut timestamps = Vec::new();
        let mut pressure_chamber_mbar = Vec::new();
        let mut pressure_transfer_mbar = Vec::new();

        // FIXME: Temporarily add some dummy data for testing.
        timestamps.push(Local::now() - chrono::Duration::hours(3));
        timestamps.push(Local::now() - chrono::Duration::hours(2));
        timestamps.push(Local::now() - chrono::Duration::hours(1));

        pressure_chamber_mbar.push(1.0e-5);
        pressure_chamber_mbar.push(2.3e-6);
        pressure_chamber_mbar.push(5.6e-7);

        pressure_transfer_mbar.push(5.0e-4);
        pressure_transfer_mbar.push(2.0e-4);
        pressure_transfer_mbar.push(8.0e-5);

        // plot style
        let plot_style = PlotStyle {
            bg_color: RGBColor(24, 24, 37),
            fg_color: RGBColor(205, 214, 244),
            transfer_color: RGBColor(249, 226, 175),
            chamber_color: RGBColor(137, 220, 235),
            mesh_major_color: RGBColor(147, 153, 178),
            mesh_minor_color: RGBColor(108, 112, 134),
            font: "sans-serif".to_string(),
        };

        Self {
            timestamps,
            pressure_chamber_mbar,
            pressure_transfer_mbar,
            plot_size,
            fname,
            ui,
            plot_style,
        }
    }

    /// Add a new measurement to the history.
    ///
    /// # Arguments
    /// * `pressure_chamber` - Pressure in the chamber.
    /// * `pressure_transfer` - Pressure in the transfer.
    ///
    /// TODO: Only keep the last N hours of measurements.
    pub fn add_measurement(
        &mut self,
        pressure_chamber: Pressure,
        pressure_transfer: Pressure,
    ) -> Result<()> {
        let ts = Local::now();
        let pc = pressure_chamber.as_millibars();
        let pt = pressure_transfer.as_millibars();

        self.timestamps.push(ts);
        self.pressure_chamber_mbar.push(pc);
        self.pressure_transfer_mbar.push(pt);

        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&self.fname)
            .expect("Opening vacuum history file must work");
        writeln!(file, "{},{},{}", ts, pc, pt).unwrap();

        self.update_plot()?;
        Ok(())
    }

    /// Update the vacuum history plot in the UI.
    pub fn update_plot(&self) -> Result<()> {
        let mut pixel_buffer =
            slint::SharedPixelBuffer::new(self.plot_size.width, self.plot_size.height);
        let size = (pixel_buffer.width(), pixel_buffer.height());

        let backend = BitMapBackend::with_buffer(pixel_buffer.make_mut_bytes(), size);
        let root = backend.into_drawing_area();

        root.fill(&self.plot_style.bg_color)?;

        let t_min = *self.timestamps.iter().min().unwrap();
        let t_max = *self.timestamps.iter().max().unwrap();
        let mut chart = ChartBuilder::on(&root)
            .margin(10)
            .set_label_area_size(LabelAreaPosition::Left, 100)
            .set_label_area_size(LabelAreaPosition::Bottom, 70)
            .build_cartesian_2d(t_min..t_max, (1.0e-10..1000.0).log_scale())?;

        chart
            .configure_mesh()
            .light_line_style(&self.plot_style.mesh_major_color)
            .label_style(
                (self.plot_style.font.as_str(), 24)
                    .into_font()
                    .color(&self.plot_style.fg_color),
            )
            .axis_style(&self.plot_style.fg_color)
            .x_desc("Time")
            .x_labels(6)
            .x_label_formatter(&|xval| xval.format("%H:%M").to_string())
            .max_light_lines(4)
            .y_desc("Pressure (mbar)")
            .y_label_formatter(&|yval| format!("{:.0e}", yval))
            .draw()?;

        chart.draw_series(LineSeries::new(
            zip(self.timestamps.clone(), self.pressure_chamber_mbar.clone()),
            &self.plot_style.chamber_color,
        ))?;
        chart.draw_series(LineSeries::new(
            zip(self.timestamps.clone(), self.pressure_transfer_mbar.clone()),
            &self.plot_style.transfer_color,
        ))?;

        // To avoid the IO failure being ignored silently, we manually call the present function
        root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");

        drop(chart);
        drop(root);
        let img = slint::Image::from_rgb8(pixel_buffer);
        let ui = self.ui.upgrade().expect("UI still exists");
        ui.global::<Logic>().set_vacuum_plot(img);

        Ok(())
    }

    /// Set the plot size in pixels.
    pub fn set_plot_size(&mut self, size: PlotSizePx) {
        self.plot_size = size;
    }
}

/// Plot size in pixels
#[derive(Clone, Debug, Default)]
pub struct PlotSizePx {
    pub width: u32,
    pub height: u32,
}

/// Plot style
struct PlotStyle {
    bg_color: RGBColor,
    fg_color: RGBColor,
    transfer_color: RGBColor,
    chamber_color: RGBColor,
    mesh_major_color: RGBColor,
    mesh_minor_color: RGBColor,
    font: String,
}
