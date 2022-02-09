use spawn_wait::SignalHandler;

fn main() {
  let mut sh = SignalHandler::default();
  println!("Try using ctrl+c.");
  loop {
    if sh.termination_pending() {
      println!("Termination signal received.");
      break;
    }
    std::thread::sleep(std::time::Duration::from_millis(100));
  }
}
