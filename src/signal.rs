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
  /// Override the default termination signals list.
  pub fn with_termination_signals(termination_signals: &[i32]) -> Self {
    let signals = SignalsInfo::new(termination_signals).unwrap();
    signals.add_signal(signal_hook::consts::SIGCHLD).unwrap();
    SignalHandler {
      signals,
      termination_signals: termination_signals.to_vec(),
    }
  }

  pub fn add_termination_signal(&mut self, signal: i32) {
    self.termination_signals.push(signal);
    self.signals.add_signal(signal).unwrap();
  }

  /// Returns true if there are unprocessed termination signals.
  ///
  /// This is useful for checking for termination signals in between different
  /// stages of processing, so that the application responds fast to signals.
  pub fn termination_pending(&mut self) -> bool {
    self
      .signals
      .pending()
      .any(|s| self.termination_signals.contains(&s))
  }
}
