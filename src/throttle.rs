use std::future::Future;

use super::*;

#[derive(Debug, Clone)]
pub struct Throttle(Arc<(&'static str, Mutex<Interval>)>);

pub fn throttle(name: &'static str, ms: u64) -> Throttle {
    Throttle::new(name, ms)
}

impl Throttle {
    pub fn new(name: &'static str, ms: u64) -> Self {
        info!("Initializing throttle {name:?} with {ms} ms minimum interval");
        let mut interval = interval(Duration::from_millis(ms));
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        Self(Arc::new((name, Mutex::new(interval))))
    }

    #[instrument(skip_all)]
    pub async fn tick(&self) {
        debug!("Waiting for throttle {:?}...", self.0 .0);
        self.0 .1.lock().await.tick().await;
        debug!("{:?} ticked!", self.0 .0);
    }

    #[instrument(skip_all)]
    pub async fn with<T, E, F: Future<Output = Result<T, E>>>(
        &self,
        f: impl FnOnce() -> F,
    ) -> Result<T, E> {
        self.tick().await;
        f().await
    }
}
