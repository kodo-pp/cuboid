use std::time::{Duration, Instant};
use std::thread;


pub struct Clock {
    last_tick: Instant,
}

impl Clock {
    pub fn new() -> Clock {
        let now = Instant::now();
        Clock { last_tick: now }
    }

    pub fn tick(&mut self, ticks_per_second: f64) -> Duration {
        if ticks_per_second.is_infinite() || ticks_per_second < 0.0 {
            panic!("Invalid `ticks_per_second` ({:?}): must be finite and positive", ticks_per_second);
        }

        let interval = Duration::from_secs_f64(ticks_per_second.recip());
        let wakeup_instant = self.last_tick + interval;
        let now = Instant::now();
        if wakeup_instant <= now {
            let result = now - self.last_tick;
            self.last_tick = now;
            return result
        }

        let sleep_duration = wakeup_instant - now;
        thread::sleep(sleep_duration);

        let after_sleep = Instant::now();
        let result = after_sleep - self.last_tick;
        self.last_tick = after_sleep;

        result
    }
}


pub struct EventsPerSecondTracker {
    last_reset: Instant,
    events_since_last_reset: u64,
}

impl EventsPerSecondTracker {
    pub fn new() -> EventsPerSecondTracker {
        EventsPerSecondTracker { last_reset: Instant::now(), events_since_last_reset: 0 }
    }

    pub fn event(&mut self) {
        self.events_since_last_reset += 1;
    }

    pub fn mean(&self) -> f64 {
        let now = Instant::now();
        (self.events_since_last_reset as f64) / (now - self.last_reset).as_secs_f64()
    }

    pub fn reset(&mut self) {
        self.last_reset = Instant::now();
        self.events_since_last_reset = 0;
    }
}


pub struct ApproximateTimer {
    interval: Duration,
    remaining: Duration,
}

impl ApproximateTimer {
    pub fn new(interval: Duration) -> ApproximateTimer {
        ApproximateTimer { interval, remaining: interval }
    }

    pub fn update(&mut self, time_delta: Duration) -> u64 {
        let difference_secs = self.remaining.as_secs_f64() - time_delta.as_secs_f64();
        let interval_secs = self.interval.as_secs_f64();
        let num_overruns = -difference_secs.div_euclid(interval_secs) as u64;
        let new_remaining = Duration::from_secs_f64(difference_secs.rem_euclid(interval_secs));
        self.remaining = new_remaining;
        num_overruns
    }
}
