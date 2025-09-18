//! Broadcast the status of the device every so often

use defmt::{error, info, warn};
use embassy_time::{Duration, Ticker};
use icd::{BcCtrlStatus, CtrlStatus};
use postcard_rpc::server::Sender;

use crate::{
    app::AppTx, baking::{GET_BAKING_SIGNAL, WATCH_BAKING}, flow::FlowMeterCtrl, valve::{
        GET_VALVE_PUMP_SIGNAL, GET_VALVE_TRANSFER_SIGNAL, WATCH_VALVE_PUMP, WATCH_VALVE_TRANSFER,
    }, vct::{GET_VCT_STATUS, WATCH_VCT_STATUS}
};

const BROADCAST_TIME_SEC: u64 = 3;

#[embassy_executor::task]
pub async fn status_broadcast(sender: Sender<AppTx>, mut flow_meter: FlowMeterCtrl) {
    info!("Status broadcast task started");
    let mut ticker = Ticker::every(Duration::from_secs(BROADCAST_TIME_SEC));

    let mut ctrl_status = CtrlStatus::default();
    let mut seq = 0u32;

    let mut baking_rec = WATCH_BAKING
        .receiver()
        .unwrap_or_else(|| {
            error!( "Could not obtain baking receiver in broadcast task");
            panic!();
        }
    );
    let mut pump_valve_rec = WATCH_VALVE_PUMP
        .receiver()
        .unwrap_or_else(|| {
            error!( "Could not obtain pump valve receiver in broadcast task");
            panic!();
        }
    );
    let mut transfer_valve_rec = WATCH_VALVE_TRANSFER
        .receiver()
        .unwrap_or_else(|| {
            error!( "Could not obtain transfer valve receiver in broadcast task");
            panic!();
        }
    );

    let mut vct_rec= WATCH_VCT_STATUS
        .receiver()
        .unwrap_or_else(|| {
            error!( "Could not obtain VCT receiver in broadcast task");
            panic!();
        }
    );



    loop {
        ticker.next().await;

        GET_BAKING_SIGNAL.signal(());
        ctrl_status.baking = baking_rec.get().await;

        ctrl_status.flow_meter = flow_meter.status();

        GET_VALVE_PUMP_SIGNAL.signal(());
        ctrl_status.pump_valve = pump_valve_rec.get().await;
        GET_VALVE_TRANSFER_SIGNAL.signal(());
        ctrl_status.transfer_valve = transfer_valve_rec.get().await;

        GET_VCT_STATUS.signal(());
        ctrl_status.vct = vct_rec.get().await;

        match sender
            .publish::<BcCtrlStatus>(seq.into(), &ctrl_status)
            .await
        {
            // Ok(_) => info!("Broadcast {} succeeded, data: {:?}", seq, defmt::Debug2Format(&ctrl_status)),
            Ok(_) => info!("Broadcast {} succeeded", seq),
            Err(_) => warn!("Broadcast {} failed", seq),
        }
        seq = seq.wrapping_add(1);
    }
}
