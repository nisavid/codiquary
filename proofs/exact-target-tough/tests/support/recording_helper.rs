use codiquary_exact_target_tough_proof::{
    AuthenticatedTargetObservation, ObservationHelper,
};
use std::sync::Mutex;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordedTarget {
    pub identity: String,
    pub length: u64,
    pub sha256: String,
}

#[derive(Debug, Default)]
pub struct RecordingHelper {
    observations: Mutex<Vec<RecordedTarget>>,
}

impl RecordingHelper {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn call_count(&self) -> usize {
        self.observations
            .lock()
            .expect("observation log lock poisoned")
            .len()
    }

    pub fn observations(&self) -> Vec<RecordedTarget> {
        self.observations
            .lock()
            .expect("observation log lock poisoned")
            .clone()
    }
}

impl ObservationHelper for RecordingHelper {
    fn observe(&self, target: AuthenticatedTargetObservation<'_>) {
        self.observations
            .lock()
            .expect("observation log lock poisoned")
            .push(RecordedTarget {
                identity: target.identity().to_owned(),
                length: target.length(),
                sha256: target.sha256().to_owned(),
            });
    }
}
