//! Reduce the Zigbee traffic of an entertainment stream.
//!
//! Every frame of an entertainment stream contains one record per
//! entertainment channel. For lights with many segments (e.g. the Hue Signe)
//! this adds up to many Zigbee messages per second, which can overload the
//! Zigbee adapter.
//!
//! [`RecordSelector`] decides which of the records of a frame are actually
//! sent. Both mechanisms are optional, and without them every record of every
//! frame is sent:
//!
//!  - a per-frame limit: at most this many records are sent for each frame.
//!    The records that were sent the longest time ago go first, so all
//!    records are updated in turn (round robin).
//!
//!  - a minimum change: records whose brightness and color barely changed
//!    since they were last sent are skipped. Skipped records are still sent
//!    again after some time, so the lights never go stale.

use std::collections::BTreeMap;

use hue::zigbee::HueEntFrameLightRecord;

/// Largest value of the 11-bit brightness field of a record
const BRIGHTNESS_MAX: u16 = 0x7FF;

/// Largest value of a 12-bit quantized color coordinate
const XY_MAX: u16 = 0xFFF;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SelectorConfig {
    /// Maximum number of records to send for each frame (`None` for no limit)
    pub max_records_per_frame: Option<usize>,

    /// Minimum change (in percent of the full range of brightness, and of
    /// each color coordinate) for a record to be sent again (`None` to always
    /// send every record)
    pub min_change_percent: Option<u8>,

    /// Records that have not been sent for this many frames are sent again,
    /// even if they have not changed
    pub refresh_frames: u64,
}

impl SelectorConfig {
    /// Is any traffic reduction configured at all?
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.max_records_per_frame.is_some() || self.min_change_percent.is_some()
    }
}

/// What we know about the last record we sent to a given address
#[derive(Clone, Copy, Debug)]
struct Sent {
    brightness: u16,
    x: u16,
    y: u16,
    frame: u64,
}

impl Sent {
    fn new(rec: &HueEntFrameLightRecord, frame: u64) -> Self {
        let (x, y) = rec.raw_xy();
        Self {
            brightness: rec.brightness(),
            x,
            y,
            frame,
        }
    }

    /// Has `rec` changed by at least the given thresholds?
    fn differs(&self, rec: &HueEntFrameLightRecord, brightness: u16, xy: u16) -> bool {
        let (x, y) = rec.raw_xy();

        self.brightness.abs_diff(rec.brightness()) >= brightness
            || self.x.abs_diff(x) >= xy
            || self.y.abs_diff(y) >= xy
    }
}

pub struct RecordSelector {
    config: SelectorConfig,
    frame: u64,
    sent: BTreeMap<u16, Sent>,
}

/// The change (at least 1) that is `percent` percent of `max`
fn scale(max: u16, percent: u8) -> u16 {
    let value = (u32::from(max) * u32::from(percent)).div_ceil(100).max(1);
    u16::try_from(value).unwrap_or(u16::MAX)
}

impl RecordSelector {
    #[must_use]
    pub const fn new(config: SelectorConfig) -> Self {
        Self {
            config,
            frame: 0,
            sent: BTreeMap::new(),
        }
    }

    /// The number of frames it takes to send all of `total` records once
    ///
    /// This is the interval (in frames) between updates of a single record.
    #[must_use]
    pub fn slices(&self, total: usize) -> usize {
        self.config
            .max_records_per_frame
            .map_or(1, |max| total.div_ceil(max.max(1)))
            .max(1)
    }

    /// Select the records of this frame that should be sent
    ///
    /// The selected records are returned in their original order.
    pub fn select(&mut self, records: Vec<HueEntFrameLightRecord>) -> Vec<HueEntFrameLightRecord> {
        if !self.config.is_active() {
            return records;
        }

        self.frame += 1;
        let now = self.frame;

        let thresholds = self
            .config
            .min_change_percent
            .map(|percent| (scale(BRIGHTNESS_MAX, percent), scale(XY_MAX, percent)));

        // The records that are due, as (frames since last sent, index)
        let mut due: Vec<(u64, usize)> = records
            .iter()
            .enumerate()
            .filter_map(|(idx, rec)| {
                let Some(prev) = self.sent.get(&rec.addr()) else {
                    // never sent before
                    return Some((u64::MAX, idx));
                };

                let age = now - prev.frame;
                let changed = thresholds.is_none_or(|(b, xy)| prev.differs(rec, b, xy));

                (changed || age >= self.config.refresh_frames).then_some((age, idx))
            })
            .collect();

        // Records that were sent the longest time ago go first
        due.sort_unstable_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));

        if let Some(max) = self.config.max_records_per_frame {
            due.truncate(max);
        }

        let mut chosen: Vec<usize> = due.into_iter().map(|(_, idx)| idx).collect();
        chosen.sort_unstable();

        let mut selected = Vec::with_capacity(chosen.len());
        for (idx, rec) in records.into_iter().enumerate() {
            if chosen.binary_search(&idx).is_ok() {
                self.sent.insert(rec.addr(), Sent::new(&rec, now));
                selected.push(rec);
            }
        }

        selected
    }
}

#[cfg(test)]
mod tests {
    use hue::zigbee::{HueEntFrameLightRecord, LightRecordMode};

    use super::{RecordSelector, SelectorConfig, scale};

    #[allow(clippy::cast_possible_truncation)]
    fn rec(addr: u16, brightness: u16, x: u16, y: u16) -> HueEntFrameLightRecord {
        let raw = [
            (x & 0xFF) as u8,
            (((x >> 8) & 0x0F) | ((y & 0x0F) << 4)) as u8,
            ((y >> 4) & 0xFF) as u8,
        ];
        HueEntFrameLightRecord::new(addr, brightness, LightRecordMode::Segment, raw)
    }

    /// A frame with `n` identical records, with addresses 0x1000.. upwards
    fn frame(n: u16) -> Vec<HueEntFrameLightRecord> {
        (0..n).map(|i| rec(0x1000 + i, 1000, 2000, 2000)).collect()
    }

    fn addrs(records: &[HueEntFrameLightRecord]) -> Vec<u16> {
        records.iter().map(HueEntFrameLightRecord::addr).collect()
    }

    fn min_change(percent: u8, refresh_frames: u64) -> SelectorConfig {
        SelectorConfig {
            max_records_per_frame: None,
            min_change_percent: Some(percent),
            refresh_frames,
        }
    }

    fn per_frame(max: usize) -> SelectorConfig {
        SelectorConfig {
            max_records_per_frame: Some(max),
            min_change_percent: None,
            refresh_frames: 0,
        }
    }

    #[test]
    fn raw_xy_round_trips() {
        let r = rec(1, 5, 0xABC, 0x123);
        assert_eq!(r.raw_xy(), (0xABC, 0x123));
    }

    #[test]
    fn scale_never_returns_zero() {
        assert_eq!(scale(0x7FF, 1), 21);
        assert_eq!(scale(0xFFF, 1), 41);
        assert_eq!(scale(0xFFF, 100), 0xFFF);
        assert_eq!(scale(10, 1), 1);
    }

    #[test]
    fn inactive_selector_sends_everything() {
        let mut sel = RecordSelector::new(SelectorConfig::default());
        for _ in 0..3 {
            assert_eq!(sel.select(frame(14)).len(), 14);
        }
    }

    #[test]
    fn first_frame_sends_every_record_when_only_min_change_is_set() {
        let mut sel = RecordSelector::new(min_change(5, 100));
        assert_eq!(sel.select(frame(14)).len(), 14);
    }

    #[test]
    fn unchanged_records_are_skipped() {
        let mut sel = RecordSelector::new(min_change(5, 100));
        sel.select(frame(4));
        assert!(sel.select(frame(4)).is_empty());
    }

    #[test]
    fn small_changes_are_skipped_and_large_ones_sent() {
        let mut sel = RecordSelector::new(min_change(5, 100));
        sel.select(frame(3));

        // 5% of 2047 is 103: record 0 changes by 10, record 1 by 200, and
        // record 2 changes color a little (5% of 4095 is 205)
        let next = vec![
            rec(0x1000, 1010, 2000, 2000),
            rec(0x1001, 1200, 2000, 2000),
            rec(0x1002, 1000, 2100, 2000),
        ];
        assert_eq!(addrs(&sel.select(next)), [0x1001]);

        let next = vec![
            rec(0x1000, 1010, 2000, 2000),
            rec(0x1001, 1200, 2000, 2000),
            rec(0x1002, 1000, 2300, 2000),
        ];
        assert_eq!(addrs(&sel.select(next)), [0x1002]);
    }

    #[test]
    fn small_changes_do_not_add_up_unnoticed() {
        // we compare against the last *sent* value, not the previous frame
        let mut sel = RecordSelector::new(min_change(5, 1000));
        sel.select(vec![rec(1, 1000, 2000, 2000)]);

        let mut sent = 0;
        for step in 1..=60 {
            let r = rec(1, 1000 + step * 10, 2000, 2000);
            sent += sel.select(vec![r]).len();
        }
        assert!(sent >= 5, "{sent}");
    }

    #[test]
    fn unchanged_records_are_refreshed() {
        let mut sel = RecordSelector::new(min_change(5, 4));
        assert_eq!(sel.select(frame(2)).len(), 2);

        let sent: Vec<usize> = (0..8).map(|_| sel.select(frame(2)).len()).collect();
        assert_eq!(sent, [0, 0, 0, 2, 0, 0, 0, 2]);
    }

    #[test]
    fn per_frame_limit_is_respected() {
        let mut sel = RecordSelector::new(per_frame(4));
        for _ in 0..20 {
            assert_eq!(sel.select(frame(14)).len(), 4);
        }
        assert_eq!(sel.select(frame(3)).len(), 3);
    }

    #[test]
    fn per_frame_limit_updates_every_record_in_turn() {
        let mut sel = RecordSelector::new(per_frame(4));
        let slices = sel.slices(14);
        assert_eq!(slices, 4);

        let mut last_sent = std::collections::BTreeMap::new();
        for frame_no in 1..=100_u64 {
            for addr in addrs(&sel.select(frame(14))) {
                if let Some(prev) = last_sent.insert(addr, frame_no) {
                    let gap = usize::try_from(frame_no - prev).unwrap();
                    assert!(gap <= slices, "record {addr:#x} waited {gap} frames");
                }
            }
        }
        assert_eq!(last_sent.len(), 14);
    }

    #[test]
    fn selected_records_keep_their_order() {
        let mut sel = RecordSelector::new(per_frame(5));
        for _ in 0..10 {
            let sent = addrs(&sel.select(frame(14)));
            assert!(sent.is_sorted());
        }
    }

    #[test]
    fn changed_records_take_priority_over_unchanged_ones() {
        let cfg = SelectorConfig {
            max_records_per_frame: Some(2),
            min_change_percent: Some(5),
            refresh_frames: 1000,
        };
        let mut sel = RecordSelector::new(cfg);

        // 4 records: send them all, 2 at a time
        assert_eq!(addrs(&sel.select(frame(4))), [0x1000, 0x1001]);
        assert_eq!(addrs(&sel.select(frame(4))), [0x1002, 0x1003]);
        assert!(sel.select(frame(4)).is_empty());

        // only record 3 changes: that is the only one sent
        let mut next = frame(4);
        next[3] = rec(0x1003, 1600, 2000, 2000);
        assert_eq!(addrs(&sel.select(next)), [0x1003]);
    }

    #[test]
    fn slices_without_limit_is_one() {
        let sel = RecordSelector::new(min_change(5, 10));
        assert_eq!(sel.slices(14), 1);
        assert_eq!(RecordSelector::new(per_frame(4)).slices(0), 1);
        assert_eq!(RecordSelector::new(per_frame(4)).slices(4), 1);
        assert_eq!(RecordSelector::new(per_frame(4)).slices(5), 2);
    }
}
