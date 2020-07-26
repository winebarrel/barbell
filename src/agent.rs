use crate::logger::QueryLog;
use crate::logger::QueryLogSender;
use crate::recorder::AgentState::AgentRunning;
use crate::recorder::AgentState::AgentStopped;
use crate::recorder::DataPoint;
use crate::recorder::DataPointSender;
use anyhow::anyhow;
use anyhow::Result;
use mysql_async::prelude::*;
use std::time;
use tokio::io::AsyncBufReadExt;

#[derive(Debug)]
pub struct Agent {
  id: u32,
  url: String,
  conn: mysql_async::Conn,
  file: String,
  rate: u32,
  key: String,
  loop_data: bool,
  force: bool,
}

impl Agent {
  pub async fn new(
    id: u32,
    url: &str,
    file: &str,
    rate: u32,
    key: &str,
    loop_data: bool,
    force: bool,
  ) -> Result<Agent> {
    let url = url.to_string();
    let mut conn = mysql_async::Conn::new(&url).await?;
    conn.ping().await?;

    Ok(Agent {
      id: id,
      url: url,
      conn: conn,
      file: file.to_string(),
      rate: rate,
      key: key.to_string(),
      loop_data: loop_data,
      force: force,
    })
  }

  pub async fn run(
    &mut self,
    _token: &str,
    rec_tx: DataPointSender,
    logger_tx: QueryLogSender,
  ) -> Result<()> {
    let mut f = tokio::fs::File::open(&self.file).await?;

    // TODO: to randomize
    f.seek(std::io::SeekFrom::Start(0)).await?;

    let mut data_points: Vec<DataPoint> = Vec::new();
    let loop_start = time::Instant::now();
    let mut next_tick = time::Duration::from_secs(1);

    loop {
      let mut reader = tokio::io::BufReader::new(&mut f);

      loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).await?;

        if n > 0 {
          let json: serde_json::Value = match serde_json::from_str(&line) {
            Ok(j) => j,
            Err(e) => {
              return Err(anyhow!("JSON parsing failed: error={}, json={}", e, line));
            }
          };

          let query = match json[&self.key].as_str() {
            Some(q) => q,
            None => {
              return Err(anyhow!(
                "JSON key not found: key={}, json={}",
                self.key,
                line
              ));
            }
          };

          println!("agent:{} query:{}", self.id, query); // TODO: delete line

          let query_start = time::Instant::now();
          self.conn.query_drop(&query).await?;
          let res_time = query_start.elapsed();

          logger_tx.send(QueryLog {
            query: query.to_string(),
            response_time: res_time,
          })?;

          if loop_start.elapsed() > next_tick {
            next_tick += time::Duration::from_secs(1);
            let dps = data_points.drain(..).collect();
            rec_tx.send((dps, AgentRunning))?;
          }

          data_points.push(DataPoint {
            timestamp: time::SystemTime::now(),
            response_time: res_time,
          });

          // TODO: delete
          tokio::time::delay_for(std::time::Duration::from_millis(100)).await;
        } else {
          // EOF
          break;
        }
      }

      if !self.loop_data {
        break;
      }

      f.seek(std::io::SeekFrom::Start(0)).await?;
    }

    if !data_points.is_empty() {
      rec_tx.send((data_points, AgentStopped))?;
    }

    Ok(())
  }
}
