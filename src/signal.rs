use signal_hook::iterator::{exfiltrator::SignalOnly, SignalsInfo};

#[derive(Debug)]
pub struct SignalHandler {
  pub(crate) signals: SignalsInfo<SignalOnly>,
  pub(crate) termination_signals: Vec<i32>,
}

impl Default for SignalHandler {
  fn default() -> Self {
    use signal_hook::consts::*;
    SignalHandler {
      signals: SignalsInfo::new(&[SIGTERM, SIGINT, SIGCHLD]).unwrap(),
      termination_signals: vec![SIGTERM, SIGINT],
    }
  }
}

impl SignalHandler {
  pub fn add_termination_signal(&mut self, signal: i32) {
    self.termination_signals.push(signal);
    self.signals.add_signal(signal).unwrap();
  }
}
