use std::collections::{BTreeMap, BTreeSet};

use hue::api::{
    Device, DeviceArchetype, DeviceProductData, Light, Metadata, RType, Resource, Room,
};
use serde_json::{Value, json};
use z2m::api::IeeeAddress;

use crate::common::fixture::{
    self, FixtureDevice, FixtureGroup, Z2mDeviceFixture, Z2mFixture, Z2mGroupFixture,
};
use crate::common::{TestBridge, TestResult};

pub mod common;

/// A Hue light that Bifrost knows (manufacturer "Signify Netherlands B.V.")
const HUE_LAMP: FixtureDevice = FixtureDevice {
    id: 1,
    friendly_name: "hue_lamp",
};

/// A Hue light of a model that Bifrost does not know (manufacturer "Philips")
const NEW_HUE_LAMP: FixtureDevice = FixtureDevice {
    id: 2,
    friendly_name: "new_hue_lamp",
};

/// A light that is not made by Hue
const OTHER_LAMP: FixtureDevice = FixtureDevice {
    id: 3,
    friendly_name: "other_lamp",
};

const ROOM: FixtureGroup = FixtureGroup {
    id: 1,
    friendly_name: "living_room",
};

/// The model of the other lamp
const OTHER_MODEL: &str = "TRADFRI bulb E27 CWS globe 806lm";

fn hue_lamp() -> Z2mDeviceFixture {
    fixture::hue::white_ambience(&HUE_LAMP)
}

fn new_hue_lamp() -> Z2mDeviceFixture {
    fixture::hue::gradient_floor_lamp(&NEW_HUE_LAMP)
}

fn other_lamp() -> Z2mDeviceFixture {
    fixture::ikea::tradfri_color(&OTHER_LAMP)
}

fn z2m_state() -> BTreeMap<String, Value> {
    Z2mFixture::new()
        .with_device(hue_lamp())
        .with_device(new_hue_lamp())
        .with_device(other_lamp())
        .with_group(
            Z2mGroupFixture::new(&ROOM)
                .with_member(&hue_lamp())
                .with_member(&new_hue_lamp())
                .with_member(&other_lamp()),
        )
        .into_state()
}

async fn device_names(bridge: &TestBridge) -> TestResult<BTreeSet<String>> {
    Ok(bridge
        .hue_client
        .get_resources::<Device>(RType::Device)
        .await?
        .into_iter()
        .map(|device| device.metadata.name)
        .collect())
}

fn only_hue() -> Value {
    json!({ "only_hue_devices": true })
}

#[tokio::test]
async fn all_devices_are_added_by_default() -> TestResult<()> {
    let mut bridge = TestBridge::start(z2m_state()).await?;
    bridge.wait_for_room(&ROOM).await?;

    let names = device_names(&bridge).await?;
    for name in ["hue_lamp", "new_hue_lamp", "other_lamp"] {
        assert!(names.contains(name), "{name} is missing: {names:?}");
    }

    Ok(())
}

#[tokio::test]
async fn only_hue_devices_leaves_out_other_makers() -> TestResult<()> {
    let mut bridge = TestBridge::start_with_z2m_config(z2m_state(), only_hue()).await?;
    bridge.wait_for_room(&ROOM).await?;

    let names = device_names(&bridge).await?;
    assert!(!names.contains("other_lamp"), "{names:?}");

    Ok(())
}

#[tokio::test]
async fn only_hue_devices_keeps_hue_devices_that_are_not_known() -> TestResult<()> {
    let mut bridge = TestBridge::start_with_z2m_config(z2m_state(), only_hue()).await?;
    bridge.wait_for_room(&ROOM).await?;

    // the model of this one is not in our database, and it is not Signify
    // by name either, but it is still recognized as a Hue light
    let names = device_names(&bridge).await?;
    assert!(names.contains("hue_lamp"), "{names:?}");
    assert!(names.contains("new_hue_lamp"), "{names:?}");

    Ok(())
}

#[tokio::test]
async fn devices_that_are_left_out_are_not_in_rooms() -> TestResult<()> {
    let mut bridge = TestBridge::start_with_z2m_config(z2m_state(), only_hue()).await?;
    bridge.wait_for_room(&ROOM).await?;

    let rooms = bridge.hue_client.get_resources::<Room>(RType::Room).await?;
    let room = rooms
        .iter()
        .find(|room| room.metadata.name == "living_room")
        .expect("the room is there");

    // the two Hue lamps are the only children
    assert_eq!(room.children.len(), 2, "{:?}", room.children);

    let hue_lamp = bridge.wait_for_device(HUE_LAMP).await?;
    assert!(room.children.iter().any(|child| child.rid == hue_lamp.id));

    Ok(())
}

#[tokio::test]
async fn devices_that_were_added_earlier_are_removed() -> TestResult<()> {
    // a device that an earlier run, without the option, left behind in the
    // state. The links are made from the address of the device, like Bifrost
    // does.
    let ieee: IeeeAddress = serde_json::from_str(&format!("\"{}\"", OTHER_LAMP.ieee_address()))?;
    let other_device = RType::Device.deterministic(&ieee);
    let other_light = RType::Light.deterministic(&ieee);

    let mut bridge = TestBridge::start_with_seed_and_z2m_config(
        z2m_state(),
        |resources| {
            let device = Device {
                product_data: DeviceProductData {
                    model_id: OTHER_MODEL.to_string(),
                    manufacturer_name: "IKEA of Sweden".to_string(),
                    product_name: "other".to_string(),
                    product_archetype: DeviceArchetype::UnknownArchetype,
                    certified: false,
                    software_version: "0.0.0".to_string(),
                    hardware_platform_type: None,
                },
                metadata: Metadata::new(DeviceArchetype::UnknownArchetype, "other_lamp"),
                services: BTreeSet::from([other_light]),
                identify: None,
                usertest: None,
            };
            resources.add(&other_device, Resource::Device(device))?;
            Ok(())
        },
        only_hue(),
    )
    .await?;
    bridge.wait_for_room(&ROOM).await?;

    let names = device_names(&bridge).await?;
    assert!(!names.contains("other_lamp"), "{names:?}");

    Ok(())
}

#[tokio::test]
async fn the_archetype_of_a_device_can_be_configured_by_name() -> TestResult<()> {
    let mut bridge = TestBridge::start_with_z2m_config(
        z2m_state(),
        json!({ "device_archetypes": { "hue_lamp": "hue_signe" } }),
    )
    .await?;

    let device = bridge.wait_for_device(HUE_LAMP).await?;
    let Resource::Device(device) = &device.obj else {
        panic!("expected a device");
    };
    assert_eq!(
        device.product_data.product_archetype,
        DeviceArchetype::HueSigne
    );

    let lights = bridge
        .hue_client
        .get_resources::<Box<Light>>(RType::Light)
        .await?;
    let light = lights
        .iter()
        .find(|light| light.metadata.name == "hue_lamp")
        .expect("the light is there");
    assert_eq!(light.metadata.archetype, DeviceArchetype::HueSigne);

    Ok(())
}

#[tokio::test]
async fn the_archetype_of_a_device_can_be_configured_by_model() -> TestResult<()> {
    let mut bridge = TestBridge::start_with_z2m_config(
        z2m_state(),
        json!({ "device_archetypes": { "tradfri bulb e27 cws globe 806lm": "candle_bulb" } }),
    )
    .await?;

    let device = bridge.wait_for_device(OTHER_LAMP).await?;
    let Resource::Device(device) = &device.obj else {
        panic!("expected a device");
    };
    assert_eq!(
        device.product_data.product_archetype,
        DeviceArchetype::CandleBulb
    );

    Ok(())
}

#[tokio::test]
async fn unknown_hue_lights_get_an_archetype_from_their_description() -> TestResult<()> {
    let mut bridge = TestBridge::start(z2m_state()).await?;

    // the description of the fixture is that of a Play gradient lightstrip
    let device = bridge.wait_for_device(NEW_HUE_LAMP).await?;
    let Resource::Device(device) = &device.obj else {
        panic!("expected a device");
    };
    assert_ne!(
        device.product_data.product_archetype,
        DeviceArchetype::UnknownArchetype
    );

    Ok(())
}
