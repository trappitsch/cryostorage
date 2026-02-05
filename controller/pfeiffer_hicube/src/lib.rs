//! Library to serve an async client to talk to the Pfeiffer HiCube pump stand via OPC UA.
//!
//! This library provides the bare bones functions that we need in order to supervise and control
//! the pump stand of the cryostorage chamber project. The driver should be split into two parts:
//! - A task that allows us asynchronously to watch specific variables of the pump.
//! - Tasks to send commands to the pump stand.
//!
//! As this is a library for one specific project, we use `anyhow` for error handling.

use std::{sync::Arc, time::Duration};

use anyhow::Result;

use opcua::{
    client::{ClientBuilder, DataChangeCallback, IdentityToken, Session},
    crypto::SecurityPolicy,
    types::{
        EndpointDescription, MessageSecurityMode, MonitoredItemCreateRequest, NodeId,
        TimestampsToReturn, UserTokenPolicy, WriteValue,
    },
};
use tokio::sync::mpsc;

mod pump_commands;
pub use pump_commands::{PumpStandState, PumpState, Variables, VentState};

use crate::pump_commands::{
    PUMP_STAND_STRING, ROUGHING_PUMP_STRING, TURBO_PUMP_STRING, VENT_STRING,
};

const SUBSCRIPTION_PUBLISH_INTERVAL_MS: u64 = 1000;

/// Pfeiffer HiCube client structure.
pub struct HiCubeClient {
    /// The opcua session after connection.
    session: Arc<Session>,
}

impl HiCubeClient {
    /// Create a new HiCubeClient.
    ///
    /// This functions tries to connect to the OPC UA server on the pump stand. If the connection
    /// fails, it will return an error.
    ///
    /// # Arguments
    /// * `ip` - The IP address of the pump stand and its port, e.g., "192.168.1.100:4840".
    pub async fn try_new_and_connect(ip: &str) -> Result<Self> {
        let endpoint_url = format!("opc.tcp://{ip}");

        let mut client = ClientBuilder::new()
            .application_name("Cryostorage HiCube Client")
            .application_uri("urn:cryostorage:hi_cube_client")
            .client()
            .expect("Builder should be in a valid state and return a client.");

        // Create the endpoint
        let endpoint: EndpointDescription = (
            endpoint_url.as_ref(),
            SecurityPolicy::None.to_str(),
            MessageSecurityMode::None,
            UserTokenPolicy::anonymous(),
        )
            .into();

        // Create the session
        // let session = client.connect_to_endpoint(endpoint, IdentityToken::Anonymous)?;
        let (session, event_loop) = client
            .connect_to_matching_endpoint(endpoint, IdentityToken::Anonymous)
            .await?;

        let ret = Self {
            session: session.clone(),
        };

        // Now we connect to the session
        tokio::task::spawn(async move {
            let handle = event_loop.spawn();
            session.wait_for_connection().await;
            handle.await.unwrap();
        });

        ret.session.wait_for_connection().await;

        Ok(ret)
    }

    /// Disconnect from the OPC UA server.
    pub async fn disconnect(&self) -> Result<()> {
        self.session.disconnect().await?;
        Ok(())
    }

    /// Get a clone of the session
    pub fn get_session(&self) -> Arc<Session> {
        self.session.clone()
    }

    /// Subscribe to value changes.
    ///
    /// This will subscribe to the value changes that are currently available in `Variables`.
    /// It will return a receiver to a `tokio::mpsc` channel that the channels will be sent to.
    /// Stopping the subscription is done by dropping the receiver.
    pub async fn subscribe(&self) -> Result<mpsc::Receiver<Variables>> {
        let (tx, rx) = mpsc::channel(100);

        // Create subscription
        let subscription_id = self
            .session
            .create_subscription(
                Duration::from_millis(SUBSCRIPTION_PUBLISH_INTERVAL_MS),
                10,
                30,
                0,
                0,
                true,
                DataChangeCallback::new(move |dv, item| {
                    match Variables::from_data_value_monitored_item(&dv, item) {
                        Ok(var) => {
                            tokio::spawn({
                                let tx = tx.clone();
                                async move {
                                    if let Err(e) = tx.send(var).await {
                                        eprintln!("Failed to send variable update: {e}");
                                    }
                                }
                            });
                        }
                        Err(e) => eprintln!("Failed to convert data value to variable: {e}"),
                    };
                }),
            )
            .await?;

        // Create the monitored items
        let items_to_create: Vec<MonitoredItemCreateRequest> = [
            PUMP_STAND_STRING.clone(),
            ROUGHING_PUMP_STRING.clone(),
            TURBO_PUMP_STRING.clone(),
            VENT_STRING.clone(),
        ]
        .iter()
        .map(|v| NodeId::new(1, v.clone()).into())
        .collect();
        let _ = self
            .session
            .create_monitored_items(subscription_id, TimestampsToReturn::Both, items_to_create)
            .await?;

        Ok(rx)
    }

    /// Write a variable to the pump stand.
    ///
    /// This will try to write a new variable to the pump stand. If it fails, it will return an
    /// error.
    pub async fn write(&mut self, variable: Variables) -> Result<()> {
        let write_value: WriteValue = variable.into();
        self.session.write(&[write_value]).await?;
        Ok(())
    }
}
