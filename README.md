[![Crates.io](https://img.shields.io/crates/v/spawn-wait)](https://crates.io/crates/spawn-wait)
[![docs.rs](https://img.shields.io/docsrs/spawn-wait)](https://docs.rs/spawn-wait)

Easily spawn and manage a set of processes.

```rust
use std::process::Command;

use spawn_wait::{ProcessSet, SignalHandler, WaitAnyResult};

fn sleep_cmd(secs: i32) -> Command {
  let mut cmd = Command::new("sleep");
  cmd.arg(secs.to_string());
  cmd
}

fn main() {
  let mut procs = ProcessSet::new();

  // You can set a limit on concurrency with ProcessSet::with_concurrency_limit(number).

  // Associate each process with a key (Hash + Eq + Clone)
  procs.add_command(3, sleep_cmd(3));
  procs.add_command(1, sleep_cmd(1));
  procs.add_command(2, sleep_cmd(2));

  let mut sh = SignalHandler::default();
  loop {
    match procs.wait_any(&mut sh) {
      WaitAnyResult::NoProcessesRunning => {
        println!("All done");
        return;
      }
      WaitAnyResult::ReceivedTerminationSignal(_) => {
        // The crate handles SIGINT for you, so you can exit cleanly
        // when the user presses ctrl-c.
        println!("Terminating");
        procs.sigint_all_and_wait(&mut sh).unwrap();
        return;
      }
      WaitAnyResult::Subprocess(id, r) => {
        println!("Process {} finished: {:?}", id, r);
      }
    }
  }
}

```
