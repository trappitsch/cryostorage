//! Module to plot the results of the simulations.
//!
//! TODO:
//! - Automatic saving of the datapoint values to a predefined file.
//! - Set proper plot attributes for color, etc. (only the ones we want to change!).
//! - Overall plot attributes?
//! - Make a plot and display it in slint
//! - Autoclean every hour or so
//! - only add new datapoints if signifantly different from the last one, or every minute
//! - Load data from file when starting the program and display the last N hours immediately.
use std::time::Duration;

use chrono::{DateTime, Local};
use plotters::style::RGBColor;

const TIME_RANGE_TO_KEEP: Duration = Duration::from_hours(24);

mod pressures;

pub use pressures::{
    PressurePlotCommands, pressure_plot_task, send_pressure_plot_cmd, send_pressure_plot_cmd_now,
};
use tokio::sync::mpsc;

use crate::logger::{LogMessage, send_log_message_now};

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

/// A measurement container that can take pressure or temperature data.
///
/// The data are named `SeriesX`, where `X is the number of the series.
/// The third data series is only used for the temperature plot and is ignored for the pressure
/// plot.
pub struct Measurements {
    plot_type: PlotType,
    timestamps: Vec<DateTime<Local>>,
    series_1: Vec<f64>,
    series_2: Vec<f64>,
    series_3: Vec<f64>, // ignored in pressure plot
    attributes_1: PlotAttributes,
    attributes_2: PlotAttributes,
    attributes_3: PlotAttributes, // ignored in pressure plot
}

impl Measurements {
    /// Create a new measurement container for a pressure plot.
    pub fn new_pressure(attr1: PlotAttributes, attr2: PlotAttributes) -> Self {
        Self {
            plot_type: PlotType::PressurePlot,
            timestamps: Vec::new(),
            series_1: Vec::new(),
            series_2: Vec::new(),
            series_3: Vec::new(), // ignored in pressure plot
            attributes_1: attr1,
            attributes_2: attr2,
            attributes_3: PlotAttributes::default(), // ignored in pressure plot
        }
    }

    /// Create a new measurement container for a temperature plot.
    pub fn new_temperature(
        attr1: PlotAttributes,
        attr2: PlotAttributes,
        attr3: PlotAttributes,
    ) -> Self {
        Self {
            plot_type: PlotType::TemperaturePlot,
            timestamps: Vec::new(),
            series_1: Vec::new(),
            series_2: Vec::new(),
            series_3: Vec::new(),
            attributes_1: attr1,
            attributes_2: attr2,
            attributes_3: attr3,
        }
    }

    /// Push a new pressure datapoint to the measurement container.
    pub fn push_pressure(&mut self, dp: PressureDataPoint) {
        if self.plot_type != PlotType::PressurePlot {
            panic!("Trying to push pressure data to a temperature plot");
        };
        self.timestamps.push(dp.ts);
        self.series_1.push(dp.chamber);
        self.series_2.push(dp.transfer);
    }

    /// Push a new temperature datapoint to the measurement container.
    pub fn push_temperature(&mut self, dp: TemperatureDataPoint) {
        if self.plot_type != PlotType::TemperaturePlot {
            panic!("Trying to push temperature data to a pressure plot");
        };
        self.timestamps.push(dp.ts);
        self.series_1.push(dp.sample);
        self.series_2.push(dp.bridge);
        self.series_3.push(dp.cooler);
    }

    /// Length of the measurement series.
    #[inline]
    pub fn len(&self) -> usize {
        self.timestamps.len()
    }

    /// Check if the measurement series is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.timestamps.is_empty()
    }

    /// Slice accessors for the timestamp series.
    #[inline]
    pub fn timestamps(&self) -> &[DateTime<Local>] {
        &self.timestamps
    }

    /// Slice accessors for the first data series.
    #[inline]
    pub fn series_1(&self) -> &[f64] {
        &self.series_1
    }

    /// Slice accessors for the second data series.
    #[inline]
    pub fn series_2(&self) -> &[f64] {
        &self.series_2
    }

    /// Slice accessors for the third data series.
    #[inline]
    pub fn series_3(&self) -> &[f64] {
        &self.series_3
    }

    /// Generic filter using indexes.
    pub fn filter_view<F>(&self, mut pred: F) -> FilteredView<'_>
    where
        F: FnMut(usize) -> bool,
    {
        let idx: Vec<usize> = (0..self.len()).filter(|&i| pred(i)).collect();
        FilteredView {
            parent: self,
            indices: idx,
        }
    }

    /// A view over a certain duration prior to now.
    pub fn last_timerange_view(&self, time_range: Duration) -> FilteredView<'_> {
        let start_time = Local::now() - time_range;
        self.filter_view(|i| {
            let t = self.timestamps[i];
            t >= start_time
        })
    }

    /// In place removal of datapoints that do not lay within the specified duration prior to now.
    pub fn retain(&mut self, time_range: Duration) {
        let keep: Vec<bool> = (0..self.len())
            .map(|i| {
                let t = self.timestamps[i];
                t >= Local::now() - time_range
            })
            .collect();

        fn compact<T>(v: &mut Vec<T>, keep: &[bool]) {
            let mut dst = 0;
            for src in 0..v.len() {
                if keep[src] {
                    v.swap(dst, src);
                    dst += 1;
                }
            }
            v.truncate(dst);
        }

        compact(&mut self.timestamps, &keep);
        compact(&mut self.series_1, &keep);
        compact(&mut self.series_2, &keep);
        if self.plot_type == PlotType::TemperaturePlot {
            compact(&mut self.series_3, &keep);
        }
    }
}

/// Non-owning, index-based view into a measurement container.
pub struct FilteredView<'a> {
    parent: &'a Measurements,
    indices: Vec<usize>,
}

impl<'a> FilteredView<'a> {
    /// Iterator yielding tuples required for the pressure data.
    pub fn iter_pressure(&self) -> impl Iterator<Item = (DateTime<Local>, f64, f64)> + '_ {
        self.indices.iter().map(move |&i| {
            (
                self.parent.timestamps[i],
                self.parent.series_1[i],
                self.parent.series_2[i],
            )
        })
    }

    /// Iterator yielding tuples required for the temperature data.
    pub fn iter_temperature(&self) -> impl Iterator<Item = (DateTime<Local>, f64, f64, f64)> + '_ {
        self.indices.iter().map(move |&i| {
            (
                self.parent.timestamps[i],
                self.parent.series_1[i],
                self.parent.series_2[i],
                self.parent.series_3[i],
            )
        })
    }

    /// Borrowed timestamp iterator.
    pub fn timestamps(&self) -> impl Iterator<Item = DateTime<Local>> + '_ {
        self.indices.iter().map(move |&i| self.parent.timestamps[i])
    }

    /// Borrowed first data series iterator.
    pub fn series_1(&self) -> impl Iterator<Item = f64> + '_ {
        self.indices.iter().map(move |&i| self.parent.series_1[i])
    }

    /// Borrowed second data series iterator.
    pub fn series_2(&self) -> impl Iterator<Item = f64> + '_ {
        self.indices.iter().map(move |&i| self.parent.series_2[i])
    }

    /// Borrowed third data series iterator.
    pub fn series_3(&self) -> impl Iterator<Item = f64> + '_ {
        self.indices.iter().map(move |&i| self.parent.series_3[i])
    }
}
