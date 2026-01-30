//! Instruments module. 
//!
//! This module handles communication with all peripherals. These are:
//! - Pfeiffer HiCube turbomolecular pump.
//! - Pfeiffer Vacuum gauge controller (two gauges, transfer and main chamber).
//! - Lakeshore temperature controller, to measure two different temperatures.
//! - Cryocooler
//! 
//! This file contains two parts:
//! - A monitoring task that needs to run in its own thread to poll instruments periodically.
//! - An async executor to send commands to instruments and change their state.
//!
//! The monitoring tasks will need to run in its own thread as the instruments are polled
//! frequently and blocking calls are needed. However, we don't want to block the entire program
//! regularly.
//!
//! The command task on the other hand can run as an async task as commands will be sent very
//! infrequently, and we will simply accept the fact that it may take half a second to set
//! something. This is acceptable for our use case. Worse case scenario will be that the interface
//! freezes until a timeout is hit. 
