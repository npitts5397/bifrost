use crate::api::{EntertainmentSegment, Position};

#[derive(Debug, Clone)]
pub struct GradientProductData {
    pub pixel_count: u32,
    pub points_capable: u32,
    pub entertainment_channel_positions: &'static [Position],
    pub entertainment_segments: &'static [EntertainmentSegment],
}

// These hardcoded channel positions are likely used by all light strips made for TVs
// Looks like |‾| (like the led strip attached to the back of a TV)
const POSITIONS_LCX001: &[Position] = &[
    Position {
        x: -0.4,
        y: 0.8,
        z: -0.4,
    },
    Position {
        x: -0.4,
        y: 0.8,
        z: 0.0,
    },
    Position {
        x: -0.4,
        y: 0.8,
        z: 0.4,
    },
    Position {
        x: 0.0,
        y: 0.8,
        z: 0.4,
    },
    Position {
        x: 0.4,
        y: 0.8,
        z: 0.4,
    },
    Position {
        x: 0.4,
        y: 0.8,
        z: 0.0,
    },
    Position {
        x: 0.4,
        y: 0.8,
        z: -0.4,
    },
];

// Obtained VIA Hue CLIP API
pub const GRADIENT_LCX001: GradientProductData = GradientProductData {
    pixel_count: 17,
    points_capable: 5,
    entertainment_channel_positions: POSITIONS_LCX001,
    entertainment_segments: &[
        EntertainmentSegment {
            start: 0,
            length: 2,
        },
        EntertainmentSegment {
            start: 2,
            length: 2,
        },
        EntertainmentSegment {
            start: 4,
            length: 3,
        },
        EntertainmentSegment {
            start: 7,
            length: 3,
        },
        EntertainmentSegment {
            start: 10,
            length: 3,
        },
        EntertainmentSegment {
            start: 13,
            length: 2,
        },
        EntertainmentSegment {
            start: 15,
            length: 2,
        },
    ],
};

// Obtained VIA Hue CLIP API, the LCX001 Position Data is reused as the two strips are identical at this time
pub const GRADIENT_LCX002: GradientProductData = GradientProductData {
    pixel_count: 20,
    points_capable: 5,
    entertainment_channel_positions: POSITIONS_LCX001,
    entertainment_segments: &[
        EntertainmentSegment {
            start: 0,
            length: 2,
        },
        EntertainmentSegment {
            start: 2,
            length: 2,
        },
        EntertainmentSegment {
            start: 4,
            length: 4,
        },
        EntertainmentSegment {
            start: 8,
            length: 4,
        },
        EntertainmentSegment {
            start: 12,
            length: 4,
        },
        EntertainmentSegment {
            start: 16,
            length: 2,
        },
        EntertainmentSegment {
            start: 18,
            length: 2,
        },
    ],
};

// These values were inferred based on the pixel length of the LCX001 and LCX002 being ~5" which would indicate the LCX003 has 24 pixels.
// The segment lengths is the best geuss based on video reviews of the model, and inferance from the LCX001 and LCX002 for 24 pixels.
// The LCX001 Position Data as it is assumed to be identical to the other two play gradient models.
pub const GRADIENT_LCX003: GradientProductData = GradientProductData {
    pixel_count: 24,
    points_capable: 5,
    entertainment_channel_positions: POSITIONS_LCX001,
    entertainment_segments: &[
        EntertainmentSegment {
            start: 0,
            length: 3,
        },
        EntertainmentSegment {
            start: 3,
            length: 3,
        },
        EntertainmentSegment {
            start: 6,
            length: 4,
        },
        EntertainmentSegment {
            start: 10,
            length: 4,
        },
        EntertainmentSegment {
            start: 14,
            length: 4,
        },
        EntertainmentSegment {
            start: 18,
            length: 3,
        },
        EntertainmentSegment {
            start: 21,
            length: 3,
        },
    ],
};

// These values are just guessed based on chrivers' commit 35e66660c8fa1e39242583e0443745ad0a8d96c3
pub const GRADIENT_LCX005: GradientProductData = GradientProductData {
    pixel_count: 18,
    points_capable: 6,
    entertainment_channel_positions: POSITIONS_LCX005,
    entertainment_segments: &[
        EntertainmentSegment {
            start: 0,
            length: 1,
        },
        EntertainmentSegment {
            start: 1,
            length: 1,
        },
        EntertainmentSegment {
            start: 2,
            length: 1,
        },
        EntertainmentSegment {
            start: 3,
            length: 1,
        },
        EntertainmentSegment {
            start: 4,
            length: 1,
        },
        EntertainmentSegment {
            start: 5,
            length: 1,
        },
        EntertainmentSegment {
            start: 6,
            length: 1,
        },
    ],
};

// These hardcoded channel positions are likely used by all light strips made for monitors
// Looks like |‾| (like the led strip attached to the back of a TV)
const POSITIONS_LCX005: &[Position] = &[
    Position {
        x: -0.4,
        y: 0.8,
        z: -0.4,
    },
    Position {
        x: -0.4,
        y: 0.8,
        z: 0.4,
    },
    Position {
        x: -0.22,
        y: 0.8,
        z: 0.4,
    },
    Position {
        x: 0.0,
        y: 0.8,
        z: 0.4,
    },
    Position {
        x: 0.22,
        y: 0.8,
        z: 0.4,
    },
    Position {
        x: 0.4,
        y: 0.8,
        z: 0.4,
    },
    Position {
        x: 0.4,
        y: 0.8,
        z: -0.4,
    },
];

pub const GRADIENT_929004610402: &GradientProductData = &GradientProductData {
    pixel_count: 18,
    points_capable: 9,
    entertainment_channel_positions: POSITIONS_929004610402,
    entertainment_segments: &[
        EntertainmentSegment {
            start: 0,
            length: 6,
        },
        EntertainmentSegment {
            start: 6,
            length: 6,
        },
        EntertainmentSegment {
            start: 12,
            length: 6,
        },
    ],
    // other hardware info:
    // 929004610402 is the 3m-long Hue Flux strip light, which has 18 cuttable parts that seem to be the smallest addressable physical unit.
    // Each cuttable part contains 6 groups of SMD LEDs.
    // Although it has 18 possible segments, it's not possible to specify more than 9 colors in the zigbee message.
};

// hardcoded channel positions likely used by all other multi-segment light strips
// Corresponds to a straight line
const POSITIONS_929004610402: &[Position] = &[
    Position {
        x: -0.4,
        y: 0.8,
        z: -0.4,
    },
    Position {
        x: 0.0,
        y: 0.8,
        z: -0.4,
    },
    Position {
        x: 0.4,
        y: 0.8,
        z: -0.4,
    },
];

/// The sizes of `groups` groups of neighbouring items that together hold
/// `count` items, as even as possible (the first groups are the larger ones)
///
/// There are never more groups than items.
#[must_use]
pub fn group_sizes(count: usize, groups: usize) -> Vec<usize> {
    if count == 0 {
        return vec![];
    }

    let groups = groups.clamp(1, count);
    let (base, extra) = (count / groups, count % groups);

    (0..groups)
        .map(|index| base + usize::from(index < extra))
        .collect()
}

/// Merge the segments of a light into at most `max` groups of neighbouring
/// segments
///
/// Each group becomes a single segment that spans the pixels of all of its
/// members. This is how the Hue API describes segments that cover several
/// pixels. Lights that have no more than `max` segments are left as they are.
#[must_use]
pub fn group_segments(segments: &[EntertainmentSegment], max: usize) -> Vec<EntertainmentSegment> {
    if max == 0 || segments.len() <= max {
        return segments.to_vec();
    }

    let mut rest = segments;
    let mut grouped = vec![];

    for size in group_sizes(segments.len(), max) {
        let (group, tail) = rest.split_at(size);
        rest = tail;

        grouped.push(EntertainmentSegment {
            start: group[0].start,
            length: group.iter().map(|segment| segment.length).sum(),
        });
    }

    grouped
}

/// One address for each of the `native` segments of a light, given the
/// addresses of the (fewer) groups of neighbouring segments they were merged
/// into by [`group_segments`]
///
/// The segments of a group share the address of their group. If there are as
/// many addresses as segments, the addresses are returned as they are.
#[must_use]
pub fn expand_addresses(addrs: &[u16], native: usize) -> Vec<u16> {
    if addrs.is_empty() || addrs.len() >= native {
        return addrs.to_vec();
    }

    addrs
        .iter()
        .zip(group_sizes(native, addrs.len()))
        .flat_map(|(addr, size)| std::iter::repeat_n(*addr, size))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{expand_addresses, group_segments, group_sizes};
    use crate::api::EntertainmentSegment;

    fn singles(count: u32) -> Vec<EntertainmentSegment> {
        (0..count)
            .map(|start| EntertainmentSegment { start, length: 1 })
            .collect()
    }

    fn pairs(segments: &[EntertainmentSegment]) -> Vec<(u32, u32)> {
        segments.iter().map(|s| (s.start, s.length)).collect()
    }

    #[test]
    fn sizes_are_as_even_as_possible() {
        assert_eq!(group_sizes(10, 3), [4, 3, 3]);
        assert_eq!(group_sizes(10, 4), [3, 3, 2, 2]);
        assert_eq!(group_sizes(7, 7), [1, 1, 1, 1, 1, 1, 1]);
        assert_eq!(group_sizes(10, 1), [10]);
    }

    #[test]
    fn sizes_never_make_more_groups_than_items() {
        assert_eq!(group_sizes(3, 10), [1, 1, 1]);
        assert_eq!(group_sizes(5, 0), [5]);
        assert_eq!(group_sizes(0, 3), Vec::<usize>::new());
    }

    #[test]
    fn sizes_always_add_up() {
        for count in 0..40 {
            for groups in 0..12 {
                assert_eq!(group_sizes(count, groups).iter().sum::<usize>(), count);
            }
        }
    }

    #[test]
    fn grouping_merges_neighbouring_segments() {
        let grouped = group_segments(&singles(10), 3);
        assert_eq!(pairs(&grouped), [(0, 4), (4, 3), (7, 3)]);
    }

    #[test]
    fn grouping_adds_up_the_lengths_of_longer_segments() {
        let segments = [
            EntertainmentSegment {
                start: 0,
                length: 2,
            },
            EntertainmentSegment {
                start: 2,
                length: 3,
            },
            EntertainmentSegment {
                start: 5,
                length: 1,
            },
            EntertainmentSegment {
                start: 6,
                length: 4,
            },
        ];

        assert_eq!(pairs(&group_segments(&segments, 2)), [(0, 5), (5, 5)]);
    }

    #[test]
    fn grouping_leaves_small_lights_alone() {
        assert_eq!(pairs(&group_segments(&singles(7), 7)), pairs(&singles(7)));
        assert_eq!(pairs(&group_segments(&singles(3), 10)), pairs(&singles(3)));
        assert_eq!(pairs(&group_segments(&singles(7), 0)), pairs(&singles(7)));
        assert_eq!(pairs(&group_segments(&[], 3)), Vec::<(u32, u32)>::new());
    }

    #[test]
    fn addresses_are_shared_within_a_group() {
        let addrs = [0xA000, 0xA001, 0xA002];

        assert_eq!(
            expand_addresses(&addrs, 10),
            [
                0xA000, 0xA000, 0xA000, 0xA000, 0xA001, 0xA001, 0xA001, 0xA002, 0xA002, 0xA002
            ]
        );
    }

    #[test]
    fn addresses_are_unchanged_without_grouping() {
        let addrs = [1, 2, 3, 4, 5, 6, 7];

        assert_eq!(expand_addresses(&addrs, 7), addrs);
        assert_eq!(expand_addresses(&addrs, 3), addrs);
        assert_eq!(expand_addresses(&[], 7), Vec::<u16>::new());
    }

    #[test]
    fn groups_and_addresses_agree() {
        // segment i of the light belongs to the group that group_segments
        // puts it in, and gets the address of that group
        let grouped = group_segments(&singles(10), 4);
        let addrs: Vec<u16> = (0..4).map(|n| 100 + n).collect();
        let expanded = expand_addresses(&addrs, 10);

        for (segment, addr) in expanded.iter().enumerate() {
            let segment = u32::try_from(segment).unwrap();
            let group = grouped
                .iter()
                .position(|g| (g.start..g.start + g.length).contains(&segment))
                .unwrap();
            assert_eq!(*addr, addrs[group]);
        }
    }
}
