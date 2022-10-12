use postgres::{Client, Error, NoTls};

mod transactions;
use crate::transactions::{delivery::*, neworder::*, payment::*, Transaction};

struct Database {
    client: Client,
}

impl Database {
    pub fn connect(params: &str) -> Result<Self, Error> {
        Ok(Self {
            client: Client::connect(params, NoTls)?,
        })
    }
}

fn main() -> Result<(), Error> {
    let mut ch_benchmark =
        Database::connect("postgresql://postgres:postgres@127.0.0.1:25432/ch_benchmark_db")?;

    (1..=10).for_each(|carrier_id| {
        dbg!("\n");
        let mut delivery = Transaction::Delivery(Delivery::new(1, 1, carrier_id));
        delivery
            .execute(ch_benchmark.client.transaction().unwrap())
            .unwrap()
    });

    (1..=10).for_each(|customer_id| {
        dbg!("\n");
        let mut neworder = Transaction::NewOrder(NewOrder::new(
            2,
            1,
            customer_id,
            10,
            1,
            vec![42, 43, 44, 45, 46],
            vec![5, 6, 7, 8, 9],
        ));
        neworder
            .execute(ch_benchmark.client.transaction().unwrap())
            .unwrap();
    });

    (1..=10).for_each(|customer_id| {
        dbg!("\n");
        let mut neworder = Transaction::NewOrder(NewOrder::new(
            2,
            1,
            customer_id,
            10,
            1,
            vec![47, 48, 49, 50, 51],
            vec![5, 6, 7, 8, 9],
        ));
        neworder
            .execute(ch_benchmark.client.transaction().unwrap())
            .unwrap();
    });

    (1..=10).for_each(|customer_id| {
        let mut payment = Transaction::Payment(Payment::new(3, 1, customer_id, 42.0, 4, 1));
        payment
            .execute(ch_benchmark.client.transaction().unwrap())
            .unwrap();
    });

    (1..=10).for_each(|customer_id| {
        let mut payment = Transaction::Payment(Payment::new(3, 1, customer_id, 42.0, 4, 1));
        payment
            .execute(ch_benchmark.client.transaction().unwrap())
            .unwrap();
    });

    (1..=10).for_each(|customer_id| {
        let mut payment = Transaction::Payment(Payment::new(3, 1, customer_id, 42.0, 4, 1));
        payment
            .execute(ch_benchmark.client.transaction().unwrap())
            .unwrap();
    });

    (1..=10).for_each(|customer_id| {
        let mut payment = Transaction::Payment(Payment::new(3, 1, customer_id, 42.0, 4, 1));
        payment
            .execute(ch_benchmark.client.transaction().unwrap())
            .unwrap();
    });

    (1..=10).for_each(|customer_id| {
        let mut payment = Transaction::Payment(Payment::new(3, 1, customer_id, 42.0, 4, 1));
        payment
            .execute(ch_benchmark.client.transaction().unwrap())
            .unwrap();
    });

    Ok(())
}
