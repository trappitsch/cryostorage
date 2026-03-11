//! Measurements module: Holds the measurements class and the Filtered View

use std::{env, io::Write, path::PathBuf, time::Duration};

use chrono::{DateTime, Local};

use crate::{
    CONFIG_FOLDER, LogMessage,
    logger::send_log_message_now,
    plots::{
        HISTORY_PRESSURE_FNAME, HISTORY_TEMPERATURE_FNAME, MAX_DURATION_BETWEEN_POINTS, MIN_LOG_DP_FACT, MIN_LOG_DT, PlotType, PressureDataPoint, TemperatureDataPoint
    },
};

/// Enum for both possible data point types.
#[derive(Clone, Debug)]
enum PossibleDataPoint {
    Pressure(PressureDataPoint),
    Temperature(TemperatureDataPoint),
}

/// A measurement container that can take pressure or temperature data.
///
/// The data are named `SeriesX`, where `X is the number of the series.
/// The third data series is only used for the temperature plot and is ignored for the pressure
/// plot.
pub struct Measurements {
    plot_type: PlotType,
    fname: PathBuf,
    timestamps: Vec<DateTime<Local>>,
    series_1: Vec<f64>,
    series_2: Vec<f64>,
    series_3: Vec<f64>, // ignored in pressure plot
    last_dp: Option<PossibleDataPoint>,
}

impl Measurements {
    /// Create a new measurement container for a pressure plot.
    pub fn new_pressure() -> Self {
        let fname = env::home_dir()
            .expect("Home directory must be known")
            .join(CONFIG_FOLDER)
            .join(HISTORY_PRESSURE_FNAME);

        // create the file with header if it does not exist
        if !fname.exists() {
            std::fs::create_dir_all(fname.parent().unwrap())
                .expect("Creating config folder must work");
            let mut file =
                std::fs::File::create(&fname).expect("Creating pressure history file must work");
            writeln!(
                file,
                "Timestamp,Chamber_pressure_mbar,Transfer_pressure_mbar"
            )
            .expect("Writing header to pressure history file must work");
        }

        Self {
            plot_type: PlotType::PressurePlot,
            fname,
            timestamps: Vec::new(),
            series_1: Vec::new(),
            series_2: Vec::new(),
            series_3: Vec::new(), // ignored in pressure plot
            last_dp: None,
        }
    }

    /// Create a new measurement container for a temperature plot.
    pub fn new_temperature() -> Self {
        let fname = env::home_dir()
            .expect("Home directory must be known")
            .join(CONFIG_FOLDER)
            .join(HISTORY_TEMPERATURE_FNAME);

        // create the file with header if it does not exist
        if !fname.exists() {
            std::fs::create_dir_all(fname.parent().unwrap())
                .expect("Creating config folder must work");
            let mut file =
                std::fs::File::create(&fname).expect("Creating pressure history file must work");
            writeln!(
                file,
                "Timestamp,Sample_temperature_K,Bridge_temperature_K,Cooler_temperature_K"
            )
            .expect("Writing header to pressure history file must work");
        }
        Self {
            plot_type: PlotType::TemperaturePlot,
            fname,
            timestamps: Vec::new(),
            series_1: Vec::new(),
            series_2: Vec::new(),
            series_3: Vec::new(),
            last_dp: None,
        }
    }

    /// Push a new pressure datapoint to the measurement container.
    pub fn push_pressure(&mut self, dp: PressureDataPoint) {
        if self.plot_type != PlotType::PressurePlot {
            panic!("Trying to push pressure data to a temperature plot");
        };

        // check if the last datapoint is too young to push a new one
        if let Some(PossibleDataPoint::Pressure(PressureDataPoint {
            ts,
            chamber,
            transfer,
        })) = &self.last_dp
            && (dp.ts - ts) < MAX_DURATION_BETWEEN_POINTS
            && ((dp.chamber / chamber - 1.0).abs() < MIN_LOG_DP_FACT)
            && ((dp.transfer / transfer - 1.0).abs() < MIN_LOG_DP_FACT)
        {
            return;
        }

        self.last_dp = Some(PossibleDataPoint::Pressure(dp.clone()));
        self.timestamps.push(dp.ts);
        self.series_1.push(dp.chamber);
        self.series_2.push(dp.transfer);

        // write to the history file
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&self.fname)
            .expect("Opening vacuum history file must work");
        if let Err(e) = writeln!(file, "{},{},{}", dp.ts, dp.chamber, dp.transfer) {
            send_log_message_now(LogMessage::new_error(&format!(
                "Failed to write pressure datapoint to history file: {e}"
            )));
        };
    }

    /// Push a new temperature datapoint to the measurement container.
    pub fn push_temperature(&mut self, dp: TemperatureDataPoint) {
        if self.plot_type != PlotType::TemperaturePlot {
            panic!("Trying to push temperature data to a pressure plot");
        };

        // check if the last datapoint is too young to push a new one
        if let Some(PossibleDataPoint::Temperature(TemperatureDataPoint {
            ts,
            sample,
            bridge,
            cooler,
        })) = &self.last_dp
            && (dp.ts - ts) < MAX_DURATION_BETWEEN_POINTS
            && ((dp.sample - sample).abs() < MIN_LOG_DT)
            && ((dp.bridge - bridge).abs() < MIN_LOG_DT)
            && ((dp.cooler - cooler).abs() < MIN_LOG_DT)
        {
            return;
        }

        self.last_dp = Some(PossibleDataPoint::Temperature(dp.clone()));
        self.timestamps.push(dp.ts);
        self.series_1.push(dp.sample);
        self.series_2.push(dp.bridge);
        self.series_3.push(dp.cooler);

        // write to the history file
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&self.fname)
            .expect("Opening temperature history file must work");
        if let Err(e) = writeln!(file, "{},{},{},{}", dp.ts, dp.sample, dp.bridge, dp.cooler) {
            send_log_message_now(LogMessage::new_error(&format!(
                "Failed to write temperature datapoint to history file: {e}"
            )));
        };
    }

    /// Length of the measurement series.
    #[inline]
    pub fn len(&self) -> usize {
        self.timestamps.len()
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
            for (src, _) in keep.iter().enumerate().take(v.len()) {
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
    /// Iterator yielding tuples required for the first data series.
    pub fn iter_series_1(&self) -> impl Iterator<Item = (DateTime<Local>, f64)> + '_ {
        self.indices
            .iter()
            .map(move |&i| (self.parent.timestamps[i], self.parent.series_1[i]))
    }

    /// Iterator yielding tuples required for the second data series.
    pub fn iter_series_2(&self) -> impl Iterator<Item = (DateTime<Local>, f64)> + '_ {
        self.indices
            .iter()
            .map(move |&i| (self.parent.timestamps[i], self.parent.series_2[i]))
    }

    /// Iterator yielding tuples required for the third data series.
    pub fn iter_series_3(&self) -> impl Iterator<Item = (DateTime<Local>, f64)> + '_ {
        self.indices
            .iter()
            .map(move |&i| (self.parent.timestamps[i], self.parent.series_3[i]))
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
