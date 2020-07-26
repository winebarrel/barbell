mod agent;
mod cli;
mod logger;
mod recorder;
mod task;

#[tokio::main]
async fn main() {
  let opts = cli::parse_opts();

  let mut t = task::Task::new(
    &opts.url,
    opts.nagents,
    &opts.files,
    &opts.log,
    opts.rate,
    &opts.key,
    opts.loop_data,
    opts.force,
  )
  .await
  .unwrap_or_else(|e| panic!("task creation failed: error={:?} options={:?}", e, &opts));

  let _report = t
    .run(opts.time, |total_count, qps, running_nagents| {
      // TODO:
      dbg!(total_count);
      dbg!(qps);
      dbg!(running_nagents);
    })
    .await
    .unwrap_or_else(|e| panic!("Task execution failed: error={:?} options={:?}", e, &opts));
}
