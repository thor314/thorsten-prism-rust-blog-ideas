use std::time::Duration;

// example of 3 software ideas:
// 1. Usage of defer pattern in rust (inspired by go api maybe!)
// 2. usage of select to implicitly await async code that may time out
// 3. great error ergonomics with anyhow!
pub async fn send_request(url: &str, metrics: &mut Metrics) -> anyhow::Result<String> {
    // defer: add a new request to tracked matrics
    // where is the defer api coming from i wonder
    let mut finish = defer(|| metrics.requests += 1);

    let request = reqwest::get(url);
    // if we get a response, do something, otherwise drop the metrics add and error timeout
    tokio::select! {
        response = request => {
            // if we get a response, clone the body and return
            let response = response?;
            let body = response.text().await?;
            Ok(body)
            // and implicitly call the closure at finish?
        }
        _ = tokio::time::sleep(Duration::from_millis(2500)) => {
            finish.abort();
            Err(anyhow::anyhow!("timeout"))
        }
    }
}

// defer uses drop to defer code excution
// defer creates a struct at `finish`, which executes its closure at drop
pub struct Deferred<T: FnOnce()> {
    task: Option<T>,
}

impl<T: FnOnce()> Deferred<T> {
    // purge the defer closure
    fn abort(&mut self) { self.task.take(); }
}

impl<T: FnOnce()> Drop for Deferred<T> {
    fn drop(&mut self) {
        if let Some(task) = self.task.take() {
            task();
        }
    }
}

pub fn defer<T: FnOnce()>(f: T) -> Deferred<T> { Deferred { task: Some(f) } }

// some dummy data or whatever
pub struct Metrics {
    requests: usize,
}
