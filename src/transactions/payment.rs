use postgres::types::ToSql;
use postgres::{Error, Transaction};

use crate::statement;
use paste::paste;

pub struct Payment {
    district_id: i32,
    warehouse_id: i32,
    customer_id: i32,
    payment_amount: f32,
}

impl Payment {
    pub fn new(district_id: i32, warehouse_id: i32, customer_id: i32, payment_amount: f32) -> Self {
        Self {
            district_id,
            warehouse_id,
            customer_id,
            payment_amount,
        }
    }

    pub fn execute(&mut self, mut transaction: Transaction) -> Result<(), Error> {
        let district_id = self.district_id;
        let warehouse_id = self.warehouse_id;
        let customer_id = self.customer_id;
        let payment_amount = self.payment_amount;

        update_warehouse(&mut transaction, &[&payment_amount, &warehouse_id])?;
        update_district(
            &mut transaction,
            &[&payment_amount, &warehouse_id, &district_id],
        )?;

        let mut customer = read_customer(
            &mut transaction,
            &[&warehouse_id, &district_id, &customer_id],
            &payment_amount,
        )?;

        match customer.c_credit.as_str() {
            "BC" => {
                customer.c_data = "default".into();
                update_customer_balance_data(
                    &mut transaction,
                    &[
                        &customer.c_balance,
                        &customer.c_ytd_payment,
                        &customer.c_payment_cnt,
                        &customer.c_data,
                        &warehouse_id,
                        &district_id,
                        &customer.c_id,
                    ],
                )?;
            }
            "GC" => {
                update_customer_balance(
                    &mut transaction,
                    &[
                        &customer.c_balance,
                        &customer.c_ytd_payment,
                        &customer.c_payment_cnt,
                        &warehouse_id,
                        &district_id,
                        &customer.c_id,
                    ],
                )?;
            }
            _ => panic!(),
        }
        transaction.commit()
    }
}

statement!(
    update,
    warehouse,
    "   UPDATE warehouse
        SET w_ytd = w_ytd + $1
        WHERE w_id = $2;"
);

statement!(
    update,
    district,
    "   UPDATE district
        SET d_ytd = d_ytd + $1
        WHERE d_w_id = $2
            AND d_id = $3;"
);

fn read_customer(
    transaction: &mut Transaction,
    params: &[&(dyn ToSql + Sync)],
    payment_amount: &f32,
) -> Result<Customer, Error> {
    let row = transaction.query_one(
        "
        SELECT
            c_id,
            c_d_id,
            c_w_id,
            c_first,
            c_middle,
            c_last,
            c_street_1,
            c_street_2,
            c_city,
            c_state,
            c_zip,
            c_phone,
            c_credit,
            c_credit_lim,
            c_discount,
            c_balance,
            c_ytd_payment,
            c_payment_cnt,
            c_since
        FROM customer
        WHERE c_w_id = $1
            AND c_d_id = $2
            AND c_id = $3;",
        params,
    )?;

    Ok(Customer {
        c_id: row.get("c_id"),
        c_payment_cnt: row.get("c_payment_cnt"),
        c_balance: row.get::<_, f32>("c_balance") - *payment_amount,
        c_ytd_payment: row.get::<_, f32>("c_ytd_payment") + *payment_amount,
        c_credit: row.get("c_credit"),
        c_data: String::new(),
    })
}

struct Customer {
    pub c_id: i32,
    pub c_payment_cnt: i32,
    pub c_balance: f32,
    pub c_ytd_payment: f32,
    pub c_credit: String,
    pub c_data: String,
}

statement!(
    update,
    customer_balance_data,
    "   UPDATE customer
        SET c_balance = $1, c_ytd_payment = $2, c_payment_cnt = $3, c_data1 = $4
        WHERE c_w_id = $5
            AND c_d_id = $6
            AND c_id = $7;"
);

statement!(
    update,
    customer_balance,
    "   UPDATE customer
        SET c_balance = $1, c_ytd_payment = $2, c_payment_cnt = $3
        WHERE c_w_id = $4
            AND c_d_id = $5
            AND c_id = $6;"
);
