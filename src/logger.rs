use anyhow::Result;
use tokio::io::AsyncWriteExt;

pub type QueryLogSender = tokio::sync::mpsc::UnboundedSender<QueryLog>;
type QueryLogReceiver = tokio::sync::mpsc::UnboundedReceiver<QueryLog>;

#[derive(Debug, serde::Serialize)]
pub struct QueryLog {
  pub query: String,
  pub response_time: tokio::time::Duration,
}

#[derive(Debug)]
pub struct Logger {
  file: tokio::fs::File,
  receiver: QueryLogReceiver,
}

impl Logger {
  pub async fn new(log: &Option<String>) -> Result<(Logger, QueryLogSender)> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    let file = if let Some(file) = log {
      tokio::fs::File::create(file).await?
    } else {
      tokio::fs::File::create("/dev/null").await?
    };

    Ok((
      Logger {
        file: file,
        receiver: rx,
      },
      tx,
    ))
  }

  pub async fn start(&mut self) -> Result<()> {
    while let Some(query_log) = self.receiver.recv().await {
      let json = serde_json::to_string(&query_log)?;
      self
        .file
        .write_all(format!("{}\n", json).as_bytes())
        .await?;
    }

    Ok(())
  }
}
