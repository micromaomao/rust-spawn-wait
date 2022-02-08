use std::process::Command;

use spawn_wait::{ProcessSet, SignalHandler, WaitAnyResult};

fn make_cmd(secs: i32) -> Command {
  let mut cmd = Command::new("sh");
  cmd.arg("-c");
  cmd.arg(format!(
    "echo Sleeping for {secs} seconds; sleep {secs}",
    secs = secs
  ));
  cmd
}

fn main() {
  let mut procs = ProcessSet::with_concurrency_limit(3);
  for i in 0..5 {
    procs.add_command((1, i), make_cmd(1));
  }
  for i in 0..5 {
    procs.add_command((2, i), make_cmd(2));
  }
  for i in 0..5 {
    procs.add_command((3, i), make_cmd(3));
  }

  let mut sh = SignalHandler::default();
  loop {
    match procs.wait_any(&mut sh) {
      WaitAnyResult::NoProcessesRunning => {
        println!("All done");
        return;
      }
      WaitAnyResult::ReceivedTerminationSignal(_) => {
        println!("Terminating");
        procs.sigint_all_and_wait(&mut sh).unwrap();
        return;
      }
      WaitAnyResult::Subprocess(id, r) => {
        println!("Process \"sleep {} # {}\" finished: {:?}", id.1, id.0, r);
      }
    }
  }
}
