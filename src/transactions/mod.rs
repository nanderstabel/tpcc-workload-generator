use postgres::Error;

pub mod delivery;
pub mod neworder;
pub mod payment;
#[macro_use]
pub mod macros;

use delivery::*;
use neworder::*;
use payment::*;

pub enum Transaction {
    Delivery(Delivery),
    NewOrder(NewOrder),
    Payment(Payment),
}

impl Transaction {
    pub fn execute(&mut self, transaction: postgres::Transaction) -> Result<(), Error> {
        match self {
            Transaction::Delivery(delivery) => delivery.execute(transaction),
            Transaction::NewOrder(neworder) => neworder.execute(transaction),
            Transaction::Payment(payment) => payment.execute(transaction),
        }
    }
}
