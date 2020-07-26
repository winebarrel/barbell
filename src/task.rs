use crate::agent::Agent;
use crate::logger::Logger;
use crate::recorder::Recorder;
use crate::recorder::Report;
use futures::future;
// use mysql_async::prelude::*;
use anyhow::Result;

#[derive(Debug)]
pub struct Task {
  nagents: u32,
  agents: Vec<Agent>,
  log: Option<String>,
}

impl Task {
  pub async fn new(
    url: &str,
    nagents: u32,
    files: &Vec<String>,
    log: &Option<String>,
    rate: u32,
    key: &str,
    loop_data: bool,
    force: bool,
  ) -> Result<Task> {
    let files_len = files.len();

    let tasks = (0..nagents).map(|i| {
      let file = &files[i as usize % files_len];
      Agent::new(i, url, file, rate, key, loop_data, force)
    });

    let agents = future::try_join_all(tasks).await?;
    Ok(Task {
      nagents: nagents,
      agents: agents,
      log: log.clone(),
    })
  }

  pub async fn run<F>(&mut self, tm: tokio::time::Duration, reporter: F) -> Result<Report>
  where
    F: Fn(u32, u32, u32),
  {
    let token = uuid::Uuid::new_v4().to_simple().to_string();
    let (mut rec, rec_tx) = Recorder::new(self.nagents);
    let (mut logger, logger_tx) = Logger::new(&self.log).await?;

    let timeout = tokio::time::timeout(
      tm,
      future::try_join3(rec.start(reporter), logger.start(), async {
        let agent_tasks = self
          .agents
          .iter_mut()
          .map(|a| a.run(&token, rec_tx.clone(), logger_tx.clone()));
        future::try_join_all(agent_tasks).await?;
        drop(rec_tx);
        drop(logger_tx);
        Ok(())
      }),
    )
    .await;

    if let Ok(result) = timeout {
      result?;
    }

    Ok(rec.report(&token))
  }
}
