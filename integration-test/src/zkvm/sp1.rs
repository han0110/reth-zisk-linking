//! A bounded pool of reusable SP1 executors sharing one transpiled program, modeled on ere's SP1
//! prover pool.

use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use anyhow::{Result, anyhow};
use crossbeam_channel::{Receiver, Sender, bounded};
use sp1_core_executor::{MinimalExecutorEnum, Program};

pub struct Sp1Pool {
    rx: Receiver<MinimalExecutorEnum>,
    tx: Sender<MinimalExecutorEnum>,
}

impl Sp1Pool {
    pub fn new(elf: &[u8], size: usize) -> Result<Self> {
        let program =
            Arc::new(Program::from(elf).map_err(|err| anyhow!("disassemble elf: {err:?}"))?);
        let (tx, rx) = bounded(size);
        for _ in 0..size {
            tx.send(MinimalExecutorEnum::new(Arc::clone(&program), false, None))
                .unwrap();
        }
        Ok(Self { rx, tx })
    }

    /// Runs the input on a free executor, blocking until one is available.
    pub fn execute(&self, input: &[u8]) -> Result<Vec<u8>> {
        let mut executor = Guard {
            executor: Some(self.rx.recv().unwrap()),
            tx: &self.tx,
        };
        executor.reset();
        executor.with_input(input);
        while executor.execute_chunk().is_some() {}
        Ok(executor.public_values_stream().clone())
    }
}

/// An executor borrowed from the pool, returned on drop.
struct Guard<'a> {
    executor: Option<MinimalExecutorEnum>,
    tx: &'a Sender<MinimalExecutorEnum>,
}

impl Deref for Guard<'_> {
    type Target = MinimalExecutorEnum;
    fn deref(&self) -> &Self::Target {
        self.executor.as_ref().unwrap()
    }
}

impl DerefMut for Guard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.executor.as_mut().unwrap()
    }
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        if let Some(executor) = self.executor.take() {
            let _ = self.tx.send(executor);
        }
    }
}
