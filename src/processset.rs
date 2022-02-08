use std::{
  collections::HashMap,
  fmt::Debug,
  hash::Hash,
  io,
  process::{Child, Command, ExitStatus},
};

use crate::{Error, SignalHandler};

#[derive(Debug)]
pub struct ProcessSet<K> {
  concurrency_limit: Option<usize>,
  queued_keys: HashMap<K, Command>,
  running_keys: HashMap<K, Child>,
  errored_keys: HashMap<K, Error>,
}

pub enum WaitAnyResult<K> {
  Subprocess(K, Result<(Child, ExitStatus), Error>),
  ReceivedTerminationSignal(i32),
  NoProcessesRunning,
}

impl<K> ProcessSet<K> {
  pub fn new() -> Self {
    ProcessSet {
      concurrency_limit: None,
      queued_keys: HashMap::new(),
      running_keys: HashMap::new(),
      errored_keys: HashMap::new(),
    }
  }

  pub fn with_concurrency_limit(limit: usize) -> Self {
    let mut n = Self::default();
    n.concurrency_limit = Some(limit);
    n
  }
}

fn take_one_from_hashmap<K: Eq + Hash + Clone, V>(hashmap: &mut HashMap<K, V>) -> Option<(K, V)> {
  let key = hashmap.keys().next();
  if let Some(key) = key {
    let key = key.clone();
    return hashmap.remove_entry(&key);
  }
  None
}

impl<K: Hash + Eq + Clone> ProcessSet<K> {
  fn spawn_processes(&mut self) {
    while !self.queued_keys.is_empty()
      && self.running_keys.len() < self.concurrency_limit.unwrap_or(usize::max_value())
    {
      let (key, mut command) = take_one_from_hashmap(&mut self.queued_keys).unwrap();
      let child_res = command.spawn();
      if let Ok(child) = child_res {
        self.running_keys.insert(key, child);
      } else {
        self
          .errored_keys
          .insert(key, Error::UnableToSpawnProcess(child_res.unwrap_err()));
      }
    }
  }

  pub fn add_command(&mut self, key: K, command: Command) {
    if self.queued_keys.contains_key(&key)
      || self.running_keys.contains_key(&key)
      || self.errored_keys.contains_key(&key)
    {
      panic!("ProcessSet::add_command: key already exists");
    }
    self.queued_keys.insert(key, command);
    self.spawn_processes();
  }

  /// Wait for any process to finish, and return the corrosponding key and resulting child (or error).
  ///
  /// Takes in a signal handler from outside which can be created with
  /// [`SignalHandler::default()`](crate::SignalHandler). This ensures that
  /// signals between waits are not missed.
  ///
  /// If multiple processes have finished, this will only return one of them.
  /// Call this function in a loop to wait for all processes to finish.
  ///
  /// If no process has finished, this will pause the current thread.
  ///
  /// If there are no processes running, this will return NoProcessesRunning.
  ///
  /// If, during the middle of waiting, the current process gets a SIGINT or SIGTERM,
  /// this will return ReceivedTerminationSignal(signal_number). More signals can be added
  /// via [`SignalHandler::add_termination_signal`](crate::SignalHandler::add_termination_signal).
  pub fn wait_any(&mut self, signal_handler: &mut SignalHandler) -> WaitAnyResult<K> {
    if let Some(res) = self.try_wait_any() {
      return res;
    }
    use signal_hook::consts::SIGCHLD;
    loop {
      let mut has_sigchld = false;
      let mut has_term = None;
      for sig in signal_handler.signals.wait() {
        if sig == SIGCHLD {
          has_sigchld = true;
        } else if signal_handler.termination_signals.contains(&sig) {
          has_term = Some(sig);
        }
      }
      if let Some(sig) = has_term {
        return WaitAnyResult::ReceivedTerminationSignal(sig);
      }
      if has_sigchld {
        if let Some(res) = self.try_wait_any() {
          return res;
        }
      }
    }
  }

  /// Non-blocking version of wait_any. If no process has finished, this will
  /// just return None.
  ///
  /// Will never return WaitAnyResult::ReceivedTerminationSignal.
  pub fn try_wait_any(&mut self) -> Option<WaitAnyResult<K>> {
    if let Some((k, e)) = take_one_from_hashmap(&mut self.errored_keys) {
      return Some(WaitAnyResult::Subprocess(k, Err(e)));
    }
    if self.running_keys.is_empty() {
      return Some(WaitAnyResult::NoProcessesRunning);
    }
    for (k, child) in self.running_keys.iter_mut() {
      let wait_res = child.try_wait();
      if let Err(e) = wait_res {
        let k = k.clone();
        let taken_k = self.running_keys.remove_entry(&k).unwrap().0;
        self.spawn_processes();
        return Some(WaitAnyResult::Subprocess(
          taken_k,
          Err(Error::WaitFailed(e)),
        ));
      }
      let wait_res = wait_res.unwrap();
      if let Some(wait_res) = wait_res {
        let k = k.clone();
        let (k, child) = self.running_keys.remove_entry(&k).unwrap();
        self.spawn_processes();
        return Some(WaitAnyResult::Subprocess(k, Ok((child, wait_res))));
      }
    }
    None
  }

  /// Kills all subprocesses.
  pub fn sigkill_all(&mut self) -> io::Result<()> {
    for (_, child) in self.running_keys.iter_mut() {
      child.kill()?;
      child.wait()?;
    }
    self.running_keys.clear();
    Ok(())
  }

  /// Send a SIGINT to all subprocesses and return immediately.
  pub fn sigint_all(&mut self) -> io::Result<()> {
    let mut k_to_remove = Vec::new();
    for (k, child) in self.running_keys.iter_mut() {
      if child.try_wait()?.is_none() {
        let pid = child.id();
        // Since we have tried to wait the child process and it is still running,
        // the pid we got must be correct.
        unsafe {
          if libc::kill(pid.try_into().unwrap(), libc::SIGINT) != 0 {
            return Err(io::Error::last_os_error());
          }
        };
      } else {
        k_to_remove.push(k.clone());
      }
    }
    for k in k_to_remove.into_iter() {
      self.running_keys.remove(&k);
    }
    Ok(())
  }

  /// Send a SIGINT to all subprocesses and wait for them to finish.
  pub fn sigint_all_and_wait(&mut self, signal_handler: &mut SignalHandler) -> io::Result<()> {
    self.sigint_all()?;
    while !self.running_keys.is_empty() {
      let wres = self.wait_any(signal_handler);
      if let WaitAnyResult::ReceivedTerminationSignal(_) = wres {
        self.sigint_all().unwrap();
      }
    }
    Ok(())
  }
}

impl<K> Default for ProcessSet<K> {
  fn default() -> Self {
    Self::new()
  }
}
