extern crate getopts;

use std::env;
use std::process;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
pub struct Options {
  pub url: String,
  pub nagents: u32,
  pub time: tokio::time::Duration,
  pub files: Vec<String>,
  pub log: Option<String>,
  pub rate: u32,
  pub key: String,
  pub loop_data: bool,
  pub random: bool,
  pub force: bool,
}

fn print_usage(program: &str, opts: getopts::Options) {
  let brief = format!("Usage: {} [options]", program);
  print!("{}", opts.usage(&brief));
}

pub fn parse_opts() -> Options {
  let args: Vec<String> = env::args().collect();
  let program = &args[0];
  let mut opts = getopts::Options::new();

  opts.optopt("u", "url", "database connection URL", "URL");
  opts.optopt("n", "nagents", "number of agents", "NAGENTS");
  opts.optopt(
    "t",
    "time",
    "test run time (sec). zero is unlimited",
    "TIME",
  );
  opts.optopt(
    "d",
    "data",
    "file path of execution queries for each agent",
    "DATA",
  );
  opts.optopt("l", "log", "file path of query log", "LOG");
  opts.optopt(
    "r",
    "rate",
    "rate limit for each agent (qps). zero is unlimited",
    "RATE",
  );
  opts.optopt("k", "key", "json key of query", "KEY");
  opts.optflag("", "no-loop", "do not loop input data");
  opts.optflag("", "force", "ignore query error");
  opts.optflag("v", "version", "print version and exit");
  opts.optflag("h", "help", "print usage and exit");

  let matches = opts
    .parse(&args[1..])
    .unwrap_or_else(|e| panic!("option parsing failed: {:?}", e));

  if matches.opt_present("h") {
    print_usage(&program, opts);
    process::exit(0)
  }

  if matches.opt_present("v") {
    println!("{}", VERSION);
    process::exit(0)
  }

  let url = matches.opt_str("u").expect("--url option missing");
  let files = matches.opt_strs("d");

  if files.len() < 1 {
    panic!("--data option missing");
  }

  let nagents = matches.opt_get("n").unwrap().unwrap_or(files.len() as u32);

  if nagents < 1 {
    panic!("'--nagents' must be >= 1")
  }

  let tm_sec = matches.opt_get_default("t", 60).unwrap();
  let log = matches.opt_str("l");
  let rate = matches.opt_get_default("r", 0).unwrap();
  let key = matches.opt_get_default("k", "query".to_string()).unwrap();
  let loop_data = !matches.opt_present("no-loop");
  let random = loop_data; // If looping, start at a random position

  Options {
    url: url,
    nagents: nagents,
    time: tokio::time::Duration::from_secs(tm_sec),
    files: files,
    log: log,
    rate: rate,
    key: key,
    loop_data: loop_data,
    random: random,
    force: matches.opt_present("force"),
  }
}
