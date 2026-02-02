//! Module to poll the Lakeshore temperature controller.

use instrumentrs::{Instrument, InstrumentInterface, TcpIpInterface};
use lakeshore_336::Lakeshore336;

