use anyhow::Result;
use std::time;

pub type DataPointSender = tokio::sync::mpsc::UnboundedSender<(Vec<DataPoint>, AgentState)>;
type DataPointReceiver = tokio::sync::mpsc::UnboundedReceiver<(Vec<DataPoint>, AgentState)>;

const REPORT_PERIOD: u64 = 1;

#[derive(Debug)]
pub struct DataPoint {
  pub timestamp: time::SystemTime,
  pub response_time: tokio::time::Duration,
}

#[derive(Debug)]
pub enum AgentState {
  AgentRunning,
  AgentStopped,
}

#[derive(Debug)]
pub struct Recorder {
  data_points: Vec<DataPoint>,
  nagents: u32,
  stopped_nagents: u32,
  receiver: DataPointReceiver,
}

#[derive(Debug)]
pub struct Report {
  pub token: String,
}

impl Recorder {
  pub fn new(nagents: u32) -> (Recorder, DataPointSender) {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    (
      Recorder {
        data_points: vec![],
        nagents: nagents,
        stopped_nagents: 0,
        receiver: rx,
      },
      tx,
    )
  }

  pub async fn start<F>(&mut self, reporter: F) -> Result<()>
  where
    F: Fn(u32, u32, u32),
  {
    let interval_start = tokio::time::Instant::now() + tokio::time::Duration::from_secs(1);
    let mut interval = tokio::time::interval_at(
      interval_start,
      tokio::time::Duration::from_secs(REPORT_PERIOD),
    );
    let mut prev_total_count = 0;

    loop {
      tokio::select! {
        data = self.receiver.recv() => {
          match data {
            Some((mut dps, agent_state)) => {
              self.data_points.append(&mut dps);

              if let AgentState::AgentStopped = agent_state {
                self.stopped_nagents += 1;
              }
            },
            None => break,
          }
        },
        _ = interval.tick() => {
          let total_count = self.data_points.len() as u32;
          let qps = total_count - prev_total_count;
          prev_total_count = total_count;
          reporter(self.data_points.len() as u32, qps, self.nagents-self.stopped_nagents);
        },
      }
    }

    Ok(())
  }

  pub fn report(&self, token: &str) -> Report {
    Report {
      token: token.to_string(),
    }
  }
}
