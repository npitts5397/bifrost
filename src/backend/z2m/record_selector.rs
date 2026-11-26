//! Reduce the Zigbee traffic of an entertainment stream.
//!
//! Every frame of an entertainment stream contains one record per
//! entertainment channel. For lights with many segments (e.g. the Hue Signe)
//! this adds up to many Zigbee messages per second.
//!
//! [`RecordSelector`] limits the number of records that are sent for each
//! frame. The records that were sent the longest time ago go first, so all
//! records are updated in turn (round robin). Without a limit, every record
//! of every frame is sent.

use std::collections::BTreeMap;

use hue::zigbee::HueEntFrameLightRecord;

pub struct RecordSelector {
    /// Maximum number of records (one per channel) to send for each frame (`None` for no limit)
    max_channels_per_frame: Option<usize>,
    frame: u64,
    /// The frame in which the record for each address was last sent
    last_sent: BTreeMap<u16, u64>,
}

impl RecordSelector {
    #[must_use]
    pub const fn new(max_channels_per_frame: Option<usize>) -> Self {
        Self {
            max_channels_per_frame,
            frame: 0,
            last_sent: BTreeMap::new(),
        }
    }

    /// The number of frames it takes to send all of `total` records once
    ///
    /// This is the interval (in frames) between updates of a single record.
    #[must_use]
    pub fn slices(&self, total: usize) -> usize {
        self.max_channels_per_frame
            .map_or(1, |max| total.div_ceil(max.max(1)))
            .max(1)
    }

    /// Select the records of this frame that should be sent
    ///
    /// The selected records are returned in their original order.
    pub fn select(&mut self, records: Vec<HueEntFrameLightRecord>) -> Vec<HueEntFrameLightRecord> {
        let Some(max) = self.max_channels_per_frame else {
            return records;
        };

        self.frame += 1;
        let now = self.frame;

        // (frames since last sent, index): records that were never sent
        // before are the oldest
        let mut by_age: Vec<(u64, usize)> = records
            .iter()
            .enumerate()
            .map(|(idx, rec)| {
                let age = self
                    .last_sent
                    .get(&rec.addr())
                    .map_or(u64::MAX, |sent| now - sent);
                (age, idx)
            })
            .collect();

        // Records that were sent the longest time ago go first
        by_age.sort_unstable_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        by_age.truncate(max);

        let mut chosen: Vec<usize> = by_age.into_iter().map(|(_, idx)| idx).collect();
        chosen.sort_unstable();

        let mut selected = Vec::with_capacity(chosen.len());
        for (idx, rec) in records.into_iter().enumerate() {
            if chosen.binary_search(&idx).is_ok() {
                self.last_sent.insert(rec.addr(), now);
                selected.push(rec);
            }
        }

        selected
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use hue::zigbee::{HueEntFrameLightRecord, LightRecordMode};

    use super::RecordSelector;

    /// A frame with `n` records, with addresses 0x1000.. upwards
    fn frame(n: u16) -> Vec<HueEntFrameLightRecord> {
        (0..n)
            .map(|i| {
                HueEntFrameLightRecord::new(0x1000 + i, 1000, LightRecordMode::Segment, [0; 3])
            })
            .collect()
    }

    fn addrs(records: &[HueEntFrameLightRecord]) -> Vec<u16> {
        records.iter().map(HueEntFrameLightRecord::addr).collect()
    }

    #[test]
    fn without_a_limit_everything_is_sent() {
        let mut sel = RecordSelector::new(None);

        for _ in 0..3 {
            assert_eq!(sel.select(frame(14)).len(), 14);
        }
    }

    #[test]
    fn a_limit_above_the_number_of_records_sends_everything() {
        let mut sel = RecordSelector::new(Some(20));

        assert_eq!(sel.select(frame(14)).len(), 14);
        assert_eq!(sel.select(frame(14)).len(), 14);
    }

    #[test]
    fn the_limit_is_respected() {
        let mut sel = RecordSelector::new(Some(4));

        for _ in 0..10 {
            assert_eq!(sel.select(frame(14)).len(), 4);
        }
    }

    #[test]
    fn records_are_sent_in_turn() {
        let mut sel = RecordSelector::new(Some(2));

        assert_eq!(addrs(&sel.select(frame(5))), [0x1000, 0x1001]);
        assert_eq!(addrs(&sel.select(frame(5))), [0x1002, 0x1003]);
        // 0x1004 was never sent, 0x1000 waited longest after that
        assert_eq!(addrs(&sel.select(frame(5))), [0x1000, 0x1004]);
        assert_eq!(addrs(&sel.select(frame(5))), [0x1001, 0x1002]);
    }

    #[test]
    fn every_record_is_updated_within_the_slices() {
        let mut sel = RecordSelector::new(Some(4));
        let slices = sel.slices(14);
        assert_eq!(slices, 4);

        let mut last_sent = BTreeMap::new();
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
        let mut sel = RecordSelector::new(Some(5));

        for _ in 0..10 {
            assert!(addrs(&sel.select(frame(14))).is_sorted());
        }
    }

    #[test]
    fn records_missing_from_a_frame_do_not_disturb_the_others() {
        let mut sel = RecordSelector::new(Some(2));

        assert_eq!(addrs(&sel.select(frame(4))), [0x1000, 0x1001]);

        // the next frame has no record for 0x1002: the others go first
        let mut next = frame(4);
        next.remove(2);
        assert_eq!(addrs(&sel.select(next)), [0x1000, 0x1003]);
    }

    #[test]
    fn slices_is_the_number_of_frames_to_update_everything() {
        let unlimited = RecordSelector::new(None);
        assert_eq!(unlimited.slices(14), 1);

        let sel = RecordSelector::new(Some(4));
        assert_eq!(sel.slices(0), 1);
        assert_eq!(sel.slices(4), 1);
        assert_eq!(sel.slices(5), 2);
        assert_eq!(sel.slices(22), 6);
    }
}
