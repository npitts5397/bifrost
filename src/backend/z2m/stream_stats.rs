//! Counters for an entertainment stream
//!
//! Tuning a stream (frame rate, records per frame, records per message, ...)
//! is only possible with numbers: how many frames came in, how many records
//! and Zigbee messages went out, and how long a record waited between two
//! updates. This module counts those, and turns them into log lines.

use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Counts {
    /// Frames received from the streaming client
    frames_offered: u64,
    /// Frames that were let through by the frame rate limit
    frames_used: u64,
    /// Light records in the used frames
    records_generated: u64,
    /// Light records that were actually sent
    records_sent: u64,
    /// Zigbee messages that were sent
    messages: u64,
}

#[allow(clippy::cast_precision_loss)]
impl Counts {
    fn per_second(count: u64, elapsed: Duration) -> f64 {
        count as f64 / elapsed.as_secs_f64().max(0.001)
    }

    fn skipped_percent(&self) -> f64 {
        if self.records_generated == 0 {
            return 0.0;
        }
        let skipped = self.records_generated.saturating_sub(self.records_sent);
        skipped as f64 * 100.0 / self.records_generated as f64
    }

    fn messages_per_used_frame(&self) -> f64 {
        if self.frames_used == 0 {
            return 0.0;
        }
        self.messages as f64 / self.frames_used as f64
    }

    fn periodic(&self, elapsed: Duration, gap: u64) -> String {
        format!(
            "stream stats ({:.0} s): frames in {:.1}/s, used {:.1}/s, records sent {:.1}/s of {:.1}/s, messages {:.1}/s, longest update gap {} frames",
            elapsed.as_secs_f64(),
            Self::per_second(self.frames_offered, elapsed),
            Self::per_second(self.frames_used, elapsed),
            Self::per_second(self.records_sent, elapsed),
            Self::per_second(self.records_generated, elapsed),
            Self::per_second(self.messages, elapsed),
            gap,
        )
    }

    fn summary(&self, elapsed: Duration, gap: u64) -> String {
        format!(
            "stream summary ({:.0} s): frames in {}, used {}; records sent {} of {} ({:.0}% skipped); messages {} ({:.1} per used frame); longest update gap {} frames",
            elapsed.as_secs_f64(),
            self.frames_offered,
            self.frames_used,
            self.records_sent,
            self.records_generated,
            self.skipped_percent(),
            self.messages,
            self.messages_per_used_frame(),
            gap,
        )
    }
}

pub struct StreamStats {
    interval: Option<Duration>,
    started: Instant,
    window_started: Instant,
    total: Counts,
    window: Counts,
    /// The longest time (in frames) a record has waited since it was last sent
    peak_age: u64,
    window_peak_age: u64,
}

impl StreamStats {
    /// Start counting. With an `interval`, [`Self::tick`] reports every time
    /// that much time has passed.
    #[must_use]
    pub const fn new(interval: Option<Duration>, now: Instant) -> Self {
        Self {
            interval,
            started: now,
            window_started: now,
            total: Counts {
                frames_offered: 0,
                frames_used: 0,
                records_generated: 0,
                records_sent: 0,
                messages: 0,
            },
            window: Counts {
                frames_offered: 0,
                frames_used: 0,
                records_generated: 0,
                records_sent: 0,
                messages: 0,
            },
            peak_age: 0,
            window_peak_age: 0,
        }
    }

    /// A frame came in from the streaming client
    pub const fn frame_offered(&mut self) {
        self.total.frames_offered += 1;
        self.window.frames_offered += 1;
    }

    /// A frame was used: `generated` records were made, `sent` of them were
    /// selected, in `messages` Zigbee messages. `gap` is the number of
    /// frames since the least recently sent record was sent.
    pub fn frame_used(&mut self, generated: usize, sent: usize, messages: usize, gap: u64) {
        for counts in [&mut self.total, &mut self.window] {
            counts.frames_used += 1;
            counts.records_generated += generated as u64;
            counts.records_sent += sent as u64;
            counts.messages += messages as u64;
        }
        self.peak_age = self.peak_age.max(gap);
        self.window_peak_age = self.window_peak_age.max(gap);
    }

    /// The line to log now, if an interval was set and has passed
    pub fn tick(&mut self, now: Instant) -> Option<String> {
        let interval = self.interval?;
        let elapsed = now.saturating_duration_since(self.window_started);
        if elapsed < interval {
            return None;
        }

        let line = self.window.periodic(elapsed, self.window_peak_age);
        self.window = Counts::default();
        self.window_peak_age = 0;
        self.window_started = now;
        Some(line)
    }

    /// The line to log when the stream is over
    #[must_use]
    pub fn summary(&self, now: Instant) -> String {
        let elapsed = now.saturating_duration_since(self.started);
        self.total.summary(elapsed, self.peak_age)
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::StreamStats;

    fn secs(start: Instant, n: u64) -> Instant {
        start + Duration::from_secs(n)
    }

    #[test]
    fn summary_reports_the_totals() {
        let t0 = Instant::now();
        let mut stats = StreamStats::new(None, t0);

        for _ in 0..10 {
            stats.frame_offered();
        }
        for _ in 0..5 {
            // 22 records made, 7 selected, in 2 messages
            stats.frame_used(22, 7, 2, 3);
        }

        assert_eq!(
            stats.summary(secs(t0, 2)),
            "stream summary (2 s): frames in 10, used 5; records sent 35 of 110 (68% skipped); \
             messages 10 (2.0 per used frame); longest update gap 3 frames"
        );
    }

    #[test]
    fn summary_of_an_empty_stream_does_not_divide_by_zero() {
        let t0 = Instant::now();
        let stats = StreamStats::new(None, t0);

        assert_eq!(
            stats.summary(t0),
            "stream summary (0 s): frames in 0, used 0; records sent 0 of 0 (0% skipped); \
             messages 0 (0.0 per used frame); longest update gap 0 frames"
        );
    }

    #[test]
    fn peak_age_is_the_largest_seen() {
        let t0 = Instant::now();
        let mut stats = StreamStats::new(None, t0);
        for age in [1, 4, 2] {
            stats.frame_used(1, 1, 1, age);
        }
        assert!(stats.summary(t0).ends_with("longest update gap 4 frames"));
    }

    #[test]
    fn no_periodic_line_without_an_interval() {
        let t0 = Instant::now();
        let mut stats = StreamStats::new(None, t0);
        stats.frame_offered();
        assert_eq!(stats.tick(secs(t0, 3600)), None);
    }

    #[test]
    fn periodic_line_waits_for_the_interval() {
        let t0 = Instant::now();
        let mut stats = StreamStats::new(Some(Duration::from_secs(10)), t0);
        stats.frame_offered();
        assert_eq!(stats.tick(secs(t0, 9)), None);
        assert!(stats.tick(secs(t0, 10)).is_some());
    }

    #[test]
    fn periodic_line_shows_rates_per_second() {
        let t0 = Instant::now();
        let mut stats = StreamStats::new(Some(Duration::from_secs(2)), t0);

        // 60 frames offered in 2 s, 40 used, 14 records of 22 sent in 4
        // messages each
        for _ in 0..60 {
            stats.frame_offered();
        }
        for _ in 0..40 {
            stats.frame_used(22, 14, 4, 3);
        }

        assert_eq!(
            stats.tick(secs(t0, 2)).as_deref(),
            Some(
                "stream stats (2 s): frames in 30.0/s, used 20.0/s, records sent 280.0/s of \
                 440.0/s, messages 80.0/s, longest update gap 3 frames"
            )
        );
    }

    #[test]
    fn each_periodic_line_covers_only_its_own_window() {
        let t0 = Instant::now();
        let mut stats = StreamStats::new(Some(Duration::from_secs(1)), t0);

        stats.frame_offered();
        stats.frame_used(10, 10, 3, 9);
        assert!(stats.tick(secs(t0, 1)).is_some());

        // the next window starts from zero, including the peak age
        stats.frame_offered();
        stats.frame_used(10, 5, 1, 2);
        let line = stats.tick(secs(t0, 2)).unwrap();
        assert!(line.contains("frames in 1.0/s"), "{line}");
        assert!(line.contains("records sent 5.0/s of 10.0/s"), "{line}");
        assert!(line.ends_with("longest update gap 2 frames"), "{line}");

        // while the summary covers the whole stream
        assert!(stats.summary(secs(t0, 2)).contains("records sent 15 of 20"));
        assert!(
            stats
                .summary(secs(t0, 2))
                .ends_with("longest update gap 9 frames")
        );
    }
}
