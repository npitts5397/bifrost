use std::collections::BTreeMap;
use std::time::Duration;

use hue::api::{Entertainment, Resource, ResourceLink};
use serde_json::{Value, json};
use tokio::time::sleep;

use crate::common::fixture::{self, FixtureDevice, Z2mFixture};
use crate::common::{TestBridge, TestResult};

pub mod common;

/// A gradient light of a model that Bifrost has no segment data for
const FLOOR_LAMP: FixtureDevice = FixtureDevice {
    id: 1,
    friendly_name: "floor_lamp",
};

/// A gradient lightstrip that Bifrost has segment data for
const STRIP: FixtureDevice = FixtureDevice {
    id: 2,
    friendly_name: "gradient_strip",
};

fn z2m_state() -> BTreeMap<String, Value> {
    Z2mFixture::new()
        .with_device(fixture::hue::gradient_floor_lamp(&FLOOR_LAMP))
        .with_device(fixture::hue::play_gradient_lightstrip(&STRIP))
        .into_state()
}

/// What a Hue light replies to the read of its segment attributes (this is
/// what a Hue Signe sends)
fn reply(count: u32) -> Value {
    json!({ "bifrost_segment_info": { "2": count, "3": 4, "4": count } })
}

async fn entertainment_service(
    bridge: &mut TestBridge,
    fixture: FixtureDevice,
) -> TestResult<ResourceLink> {
    let device = bridge.wait_for_device(fixture).await?;
    let Resource::Device(device) = &device.obj else {
        panic!("expected a device");
    };

    Ok(*device
        .entertainment_service()
        .expect("light has an entertainment service"))
}

async fn segment_count(bridge: &TestBridge, service: ResourceLink) -> TestResult<usize> {
    let entertainment: Entertainment = bridge.hue_client.get_resource(service).await?;

    Ok(entertainment
        .segments
        .map_or(0, |segments| segments.segments.len()))
}

/// Wait until the number of segments is `expected` (or give up after a while)
async fn wait_for_segment_count(
    bridge: &TestBridge,
    service: ResourceLink,
    expected: usize,
) -> TestResult<usize> {
    let mut count = segment_count(bridge, service).await?;

    for _ in 0..20 {
        if count == expected {
            break;
        }
        sleep(Duration::from_millis(50)).await;
        count = segment_count(bridge, service).await?;
    }

    Ok(count)
}

#[tokio::test]
async fn a_guessed_segment_count_is_replaced_by_the_one_of_the_light() -> TestResult<()> {
    let mut bridge = TestBridge::start(z2m_state()).await?;
    let service = entertainment_service(&mut bridge, FLOOR_LAMP).await?;

    // we only have the segments of a Hue Flux lightstrip to go by
    let guess = segment_count(&bridge, service).await?;
    assert_eq!(guess, 3);

    bridge
        .z2m
        .publish_topic("floor_lamp".to_string(), reply(10))?;

    assert_eq!(wait_for_segment_count(&bridge, service, 10).await?, 10);

    Ok(())
}

#[tokio::test]
async fn known_segment_data_is_kept_when_the_light_reports_something_else() -> TestResult<()> {
    let mut bridge = TestBridge::start(z2m_state()).await?;
    let service = entertainment_service(&mut bridge, STRIP).await?;

    let known = segment_count(&bridge, service).await?;
    assert_eq!(known, 7);

    bridge
        .z2m
        .publish_topic("gradient_strip".to_string(), reply(10))?;

    // the reply is handled, but does not change anything
    sleep(Duration::from_millis(300)).await;
    assert_eq!(segment_count(&bridge, service).await?, known);

    Ok(())
}

#[tokio::test]
async fn an_implausible_segment_count_is_ignored() -> TestResult<()> {
    let mut bridge = TestBridge::start(z2m_state()).await?;
    let service = entertainment_service(&mut bridge, FLOOR_LAMP).await?;

    for count in [0, 1, 255] {
        bridge
            .z2m
            .publish_topic("floor_lamp".to_string(), reply(count))?;
    }

    sleep(Duration::from_millis(300)).await;
    assert_eq!(segment_count(&bridge, service).await?, 3);

    Ok(())
}
