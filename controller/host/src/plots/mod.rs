//! Module to plot the results of the simulations.
//!
//! TODO:
//! - Automatic saving of the datapoint values to a predefined file.
//! - Autoclean every hour or so
//! - only add new datapoints if signifantly different from the last one, or every minute
//! - Load data from file when starting the program and display the last N hours immediately.
use std::time::Duration;

use chrono::{DateTime, Local};
use plotters::prelude::*;

mod measurements;
mod pressures;
mod temperatures;

pub use measurements::*;
pub use pressures::{PressurePlotCommands, pressure_plot_task, send_pressure_plot_cmd_now};
pub use temperatures::{
    TemperaturePlotCommands, send_temperature_plot_cmd_now, temperature_plot_task,
};

pub const PLOT_STYLE: PlotStyle = PlotStyle {
    bg_color: RGBColor(24, 24, 37),
    fg_color: RGBColor(205, 214, 244),
    mesh_major_color: RGBColor(69, 71, 90), // surface1
    mesh_minor_color: RGBColor(49, 50, 68), //surface0
    font: "sans-serif",
    transfer_color: RGBColor(249, 226, 175), // yellow
    chamber_color: RGBColor(137, 220, 235),  // sky
    sample_color: RGBColor(245, 224, 220),   // rosewater
    bridge_color: RGBColor(166, 227, 161),   // green
    cooler_color: RGBColor(137, 220, 235),   // sky
};

const TIME_RANGE_TO_KEEP: Duration = Duration::from_hours(24);

/// One datapoint for the pressure plot.
///
/// Unit used: mbar.
#[derive(Clone, Debug)]
pub struct PressureDataPoint {
    /// Time stamp of this datapoint.
    pub ts: DateTime<Local>,
    /// Chamber pressure in mbar.
    pub chamber: f64,
    /// Transfer pressure in mbar.
    pub transfer: f64,
}

/// One datapoint for the temperature plot.
///
/// Unit used: K.
#[derive(Clone, Debug)]
pub struct TemperatureDataPoint {
    /// Time stamp of this datapoint.
    pub ts: DateTime<Local>,
    /// Sample temperature in K.
    pub sample: f64,
    /// Bridge temperature in K.
    pub bridge: f64,
    /// Cooler temperature in K.
    pub cooler: f64,
}

/// What type of plot do we have?
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlotType {
    /// Pressure plot
    PressurePlot,
    /// Temperature plot
    TemperaturePlot,
}

/// Attributes of a given function to plot.
#[derive(Clone, Debug, Default)]
pub struct PlotAttributes {
    /// Name of the function to plot.
    pub name: String,
    pub color: RGBColor,
}

/// Holds the plot size in pixels.
#[derive(Clone, Debug)]
pub struct PlotSizePx {
    pub width: u32,
    pub height: u32,
}

/// Plot style
pub struct PlotStyle {
    pub bg_color: RGBColor,
    pub fg_color: RGBColor,
    pub mesh_major_color: RGBColor,
    pub mesh_minor_color: RGBColor,
    pub font: &'static str,
    pub transfer_color: RGBColor,
    pub chamber_color: RGBColor,
    pub sample_color: RGBColor,
    pub bridge_color: RGBColor,
    pub cooler_color: RGBColor,
}
