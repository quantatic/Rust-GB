use rodio::{Sample, Source};

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub fn samples_queue<S: Sample>(
    channels: u16,
    sample_rate: u32,
) -> (SamplesQueueInput<S>, SamplesQueueOutput<S>) {
    let samples = Arc::default();

    let input = SamplesQueueInput {
        next_samples: Arc::clone(&samples),
    };

    let output = SamplesQueueOutput {
        next_samples: Arc::clone(&samples),
        last_output: S::zero_value(),
        channels,
        sample_rate,
        max_queued_samples: usize::try_from(sample_rate).unwrap() / 10,
    };
    (input, output)
}

#[derive(Clone)]
pub struct SamplesQueueInput<S: Sample> {
    next_samples: Arc<Mutex<VecDeque<S>>>,
}

impl<S: Sample> SamplesQueueInput<S> {
    pub fn append(&self, values: impl IntoIterator<Item = S>) {
        let mut next_samples = self.next_samples.lock().unwrap();

        next_samples.extend(values);
    }
}

pub struct SamplesQueueOutput<S: Sample> {
    next_samples: Arc<Mutex<VecDeque<S>>>,
    last_output: S,
    channels: u16,
    sample_rate: u32,
    max_queued_samples: usize,
}

impl<S: Sample> Iterator for SamplesQueueOutput<S> {
    type Item = S;

    fn next(&mut self) -> Option<S> {
        let mut next_samples = self.next_samples.lock().unwrap();

        while next_samples.len() > self.max_queued_samples * usize::from(self.channels) {
            for _ in 0..self.channels {
                next_samples.pop_front();
            }
        }

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
