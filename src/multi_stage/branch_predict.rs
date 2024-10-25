use std::collections::HashMap;

use log::warn;

use super::cpu::PredictPolicy;

/// Branch history table
pub struct BHT {
    inner: HashMap<u64, u8>, // pc -> taken
    predict_policy: PredictPolicy,
}

/// Branch target buffer
pub struct BTB {
    inner: HashMap<u64, u64>, // pc -> branch target address
}

/// Return address stack
#[derive(Debug)]
pub struct RAS {
    inner: Vec<u64>,
}

impl BHT {
    pub fn new(predict_policy: PredictPolicy) -> Self {
        Self {
            inner: HashMap::new(),
            predict_policy,
        }
    }

    pub fn init_pc_predict(&mut self, pc: u64) -> u8 {
        let init_predict = match self.predict_policy {
            PredictPolicy::OneBitPredict => {
                0 // Initially not taken
            }
            PredictPolicy::TwoBitsPredict => {
                0b01 // Initially not taken but in an unstable FSM state
            }
        };
        self.inner.insert(pc, init_predict);
        init_predict
    }

    /// Called in Fetch phase
    pub fn predict(&mut self, pc: u64) -> bool {
        let result = self
            .inner
            .get(&pc)
            .cloned()
            .unwrap_or_else(|| self.init_pc_predict(pc));

        match self.predict_policy {
            PredictPolicy::OneBitPredict => {
                assert!(result == 0 || result == 1);
                result == 1
            }
            PredictPolicy::TwoBitsPredict => {
                assert!(result == 0b00 || result == 0b01 || result == 0b10 || result == 0b11);
                match result {
                    0b00 | 0b01 => false, // branch not taken
                    0b10 | 0b11 => true,  // branch taken
                    _ => unreachable!(),
                }
            }
        }
    }

    /// Called by CPU with Exec phase result
    pub fn update_with_result(&mut self, pc: u64, taken: bool) {
        match self.predict_policy {
            PredictPolicy::OneBitPredict => {
                self.inner.insert(pc, if taken { 1 } else { 0 });
            }
            PredictPolicy::TwoBitsPredict => {
                let original_state = *self
                    .inner
                    .get(&pc)
                    .expect(format!("Not initialized: {pc:#x}").as_str());
                let new_state = match (original_state, taken) {
                    (0b00, false) => 0b00,
                    (0b00, true) => 0b01,
                    (0b01, false) => 0b00,
                    (0b01, true) => 0b11,
                    (0b11, false) => 0b10,
                    (0b11, true) => 0b11,
                    (0b10, false) => 0b00,
                    (0b10, true) => 0b11,
                    _ => unreachable!(),
                };
                self.inner.insert(pc, new_state);
            }
        }
    }
}

impl BTB {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Called in Fetch phase
    pub fn query_target(&self, pc: u64) -> Option<u64> {
        self.inner.get(&pc).cloned()
    }

    /// Called by CPU with Exec phase result
    pub fn add_entry(&mut self, pc: u64, target: u64, is_jalr: bool) {
        let old_target = self.inner.insert(pc, target);
        // sanity check
        if is_jalr {
            /* Do nothing */
        } else {
            if let Some(old_target) = old_target {
                assert!(old_target == target)
            }
        }
    }
}

impl RAS {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn push(&mut self, ra: u64) {
        self.inner.push(ra);
    }

    pub fn pop(&mut self) -> Option<u64> {
        self.inner.pop()
    }
}
