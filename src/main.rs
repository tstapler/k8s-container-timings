#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate anyhow;
use futures::{StreamExt, TryStreamExt};
use kube::{
    client::APIClient,
    config,
    api::{Resource, Meta, ListParams, WatchEvent},
    runtime::{Informer}
};
use k8s_openapi::{
    api::core::v1::{Pod},
};
use chrono::{DateTime, Utc};


#[derive(Serialize, Clone)]
pub struct Entry {
    container: String,
    name: String,
    version: String,
}

#[derive(Serialize, Clone)]
pub struct TimeEntry {
    pod_name: String,
    time: DateTime<Utc>
}

use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env::set_var("RUST_LOG", "info,kube=debug");
    env_logger::init();
    let config = config::load_kube_config().await?;
    let client = APIClient::new(config);
    let r = Resource::all::<Pod>();
    let inf = Informer::<Pod>::new(client, ListParams::default(), r);
    loop {
        let mut pods = inf.poll().await?.boxed();

        while let Some(event) = pods.try_next().await? {
            handle_pod(event)?;
        }
    }
}

fn startup_time(pod: Pod) -> anyhow::Result<i64> {
    if pod.status.is_none() {
        return Err(anyhow!("No pod status"));
    }

    let status = pod.status.unwrap();
    let mut conditions = HashMap::new();
    for c in status.conditions.unwrap().into_iter() {
        conditions.insert(c.type_, c.last_transition_time.unwrap());
    }
    if conditions.contains_key("Ready") && conditions.contains_key("PodScheduled") {
        let scheduled_time: DateTime<Utc> = conditions.get("PodScheduled").unwrap().0;
        let ready_time: DateTime<Utc> = conditions.get("Ready").unwrap().0;
        let duration = ready_time.signed_duration_since(scheduled_time);
        if duration.num_seconds() > 600 {
            return Err(anyhow!("Pod took too long to startup"));
        }
        return Ok(duration.num_seconds());
    }
    return Err(anyhow!("Not enough information to get startup time"));
}

use std::collections::HashMap;
// This function lets the app handle an event from kube
fn handle_pod(ev: WatchEvent<Pod>) -> anyhow::Result<()> {
    match ev {
        WatchEvent::Added(pod) => {
            let name = Meta::name(&pod);
            if let Ok(seconds)  = startup_time(pod) {
                info!("{} it took {:?} seconds", name, seconds);
            }
        }
        WatchEvent::Modified(pod) => {
            let name = Meta::name(&pod);
            if let Ok(seconds)  = startup_time(pod) {
                info!("{} it took {:?} seconds", name, seconds);
            }
        }
        _ => {}
    }
    Ok(())
}