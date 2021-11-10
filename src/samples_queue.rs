use rodio::{Sample, Source};

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub fn samples_queue<S: Sample>(
    channels: u16,
    sample_rate: u32,
) -> (Arc<SamplesQueueInput<S>>, SamplesQueueOutput<S>) {
    let input = Arc::new(SamplesQueueInput {
        next_samples: Mutex::default(),
    });
    let output = SamplesQueueOutput {
        input: Arc::clone(&input),
        last_output: S::zero_value(),
        channels,
        sample_rate,
    };
    (input, output)
}

pub struct SamplesQueueInput<S: Sample> {
    next_samples: Mutex<VecDeque<S>>,
}

impl<S: Sample> SamplesQueueInput<S> {
    pub fn append(&self, values: impl IntoIterator<Item = S>) {
        self.next_samples.lock().unwrap().extend(values);
    }
}

pub struct SamplesQueueOutput<S: Sample> {
    input: Arc<SamplesQueueInput<S>>,
    last_output: S,
    channels: u16,
    sample_rate: u32,
}

impl<S: Sample> Iterator for SamplesQueueOutput<S> {
    type Item = S;

    fn next(&mut self) -> Option<S> {
        let mut next_samples = self.input.next_samples.lock().unwrap();

        if let Some(next_output) = next_samples.pop_front() {
            self.last_output = next_output
        }

        Some(self.last_output)
    }
}

impl<S: Sample> Source for SamplesQueueOutput<S> {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}
