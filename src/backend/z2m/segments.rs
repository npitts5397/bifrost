//! Ask Hue lights how many segments they have.
//!
//! Multi-segment Hue lights (gradient lightstrips, the Hue Signe, ...) report
//! their segment count in an attribute of the Philips entertainment cluster
//! (0xFC01). `Zigbee2MQTT` can read that attribute for us, and publish the result
//! as part of the state of the light, if we ask for it with `state_property`.
//!
//! The attribute is documented as "probably segment count"; the other
//! attributes we ask for are only logged.

use serde_json::{Value, json};

use hue::zigbee::{EntertainmentZigbeeStream, PHILIPS_HUE_ZIGBEE_VENDOR_ID};

/// Name of the state property that `Zigbee2MQTT` publishes the reply under
pub const PROPERTY: &str = "bifrost_segment_info";

/// Attributes to read: max segment count, "gradient related", segment count
const ATTRIBUTES: [u8; 3] = [2, 3, 4];

/// The attribute that holds the segment count
const SEGMENT_COUNT_ATTRIBUTE: &str = "4";

/// Lights that report more segments than this are not believed
const MAX_PLAUSIBLE_SEGMENTS: usize = 32;

/// The request to send to the `set` topic of a light
#[must_use]
pub fn request() -> Value {
    json!({
        "read": {
            "cluster": EntertainmentZigbeeStream::CLUSTER,
            "attributes": ATTRIBUTES,
            "options": { "manufacturerCode": PHILIPS_HUE_ZIGBEE_VENDOR_ID },
            "state_property": PROPERTY,
        }
    })
}

/// The segment count in the reply, if there is a plausible one
///
/// The reply is an object with the attribute numbers (as strings) as keys.
#[must_use]
pub fn reported_count(info: &Value) -> Option<usize> {
    let count = usize::try_from(info.get(SEGMENT_COUNT_ATTRIBUTE)?.as_u64()?).ok()?;

    (2..=MAX_PLAUSIBLE_SEGMENTS)
        .contains(&count)
        .then_some(count)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{PROPERTY, reported_count, request};

    #[test]
    fn request_reads_the_philips_entertainment_cluster() {
        let req = request();

        assert_eq!(req["read"]["cluster"], 0xFC01);
        assert_eq!(req["read"]["attributes"], json!([2, 3, 4]));
        assert_eq!(req["read"]["options"]["manufacturerCode"], 0x100B);
        assert_eq!(req["read"]["state_property"], PROPERTY);
    }

    #[test]
    fn count_is_taken_from_attribute_4() {
        // reply read from a Hue Signe
        assert_eq!(reported_count(&json!({"2": 10, "3": 4, "4": 10})), Some(10));
        assert_eq!(reported_count(&json!({"2": 10, "3": 4, "4": 7})), Some(7));
    }

    #[test]
    fn implausible_or_missing_counts_are_ignored() {
        assert_eq!(reported_count(&json!({"4": 0})), None);
        assert_eq!(reported_count(&json!({"4": 1})), None);
        assert_eq!(reported_count(&json!({"4": 255})), None);
        assert_eq!(reported_count(&json!({"2": 10})), None);
        assert_eq!(reported_count(&json!({"4": "ten"})), None);
        assert_eq!(reported_count(&json!(null)), None);
        assert_eq!(reported_count(&json!([10])), None);
    }
}
