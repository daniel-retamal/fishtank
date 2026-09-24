use crate::economy::Money;
use crate::ledger::{Direction, Flow};

use super::App;

impl App {
    pub fn earn(&mut self, amount: impl Into<Money>, flow: Flow) {
        let amount = amount.into();
        if amount == 0 {
            return;
        }
        self.purse.earn(amount);
        self.ledger.record(Direction::In, flow, amount);
    }

    pub fn pay(&mut self, cost: impl Into<Money>, flow: Flow) -> bool {
        let cost = cost.into();
        if !self.purse.spend(cost) {
            return false;
        }
        self.ledger.record(Direction::Out, flow, cost);
        true
    }
}
