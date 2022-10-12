use postgres::types::ToSql;
use postgres::{Error, Transaction};

use crate::statement;
use paste::paste;

pub struct Payment {
    district_id: i32,
    warehouse_id: i32,
    customer_id: i32,
    payment_amount: f32,
    customer_district_id: i32,
    customer_warehouse_id: i32,
}

impl Payment {
    pub fn new(
        district_id: i32,
        warehouse_id: i32,
        customer_id: i32,
        payment_amount: f32,
        customer_district_id: i32,
        customer_warehouse_id: i32,
    ) -> Self {
        Self {
            district_id,
            warehouse_id,
            customer_id,
            payment_amount,
            customer_district_id,
            customer_warehouse_id,
        }
    }

    pub fn execute(&mut self, mut transaction: Transaction) -> Result<(), Error> {
        let district_id = self.district_id;
        let warehouse_id = self.warehouse_id;
        let customer_id = self.customer_id;
        let payment_amount = self.payment_amount;
        let customer_district_id = self.customer_district_id;
        let customer_warehouse_id = self.customer_warehouse_id;

        update_warehouse(&mut transaction, &[&payment_amount, &warehouse_id])?;
        let warehouse_name = read_warehouse_name(&mut transaction, &[&warehouse_id])?;
        update_district(
            &mut transaction,
            &[&payment_amount, &warehouse_id, &district_id],
        )?;
        let district_name = read_district_name(&mut transaction, &[&warehouse_id, &district_id])?;

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

        // insert_history(
        //     &mut transaction,
        //     &[
        //         &customer_district_id,
        //         &customer_warehouse_id,
        //         &customer.c_id,
        //         &district_id,
        //         &warehouse_id,
        //         &payment_amount,
        //         &format!("{warehouse_name} ${district_name}"),
        //     ],
        // )?;

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
    read,
    warehouse_name,
    "   SELECT
            w_name
        FROM warehouse
        WHERE w_id = $1;",
    String,
    "w_name"
);

statement!(
    update,
    district,
    "   UPDATE district
        SET d_ytd = d_ytd + $1
        WHERE d_w_id = $2
            AND d_id = $3;"
);

statement!(
    read,
    district_name,
    "   SELECT
            d_name
        FROM district
        WHERE d_w_id = $1
            AND d_id = $2;",
    String,
    "d_name"
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
        // c_d_id: row.get("c_d_id"),
        // c_w_id: row.get("c_w_id"),
        c_payment_cnt: row.get("c_payment_cnt"),
        // c_delivery_cnt: -1,
        // c_since: row.get("c_since"),
        // c_discount: row.get("c_discount"),
        // c_credit_lim: row.get("c_credit_lim"),
        c_balance: row.get::<_, f32>("c_balance") - *payment_amount,
        c_ytd_payment: row.get::<_, f32>("c_ytd_payment") + *payment_amount,
        c_credit: row.get("c_credit"),
        // c_last: row.get("c_last"),
        // c_first: row.get("c_first"),
        // c_street_1: row.get("c_street_1"),
        // c_street_2: row.get("c_street_2"),
        // c_city: row.get("c_city"),
        // c_state: row.get("c_state"),
        // c_zip: row.get("c_zip"),
        // c_phone: row.get("c_phone"),
        // c_middle: row.get("c_middle"),
        c_data: String::new(),
    })
}

struct Customer {
    pub c_id: i32,
    // pub c_d_id: i32,
    // pub c_w_id: i32,
    pub c_payment_cnt: i32,
    // pub c_delivery_cnt: i32,
    // pub c_since: String,
    // pub c_discount: f32,
    // pub c_credit_lim: f32,
    pub c_balance: f32,
    pub c_ytd_payment: f32,
    pub c_credit: String,
    // pub c_last: String,
    // pub c_first: String,
    // pub c_street_1: String,
    // pub c_street_2: String,
    // pub c_city: String,
    // pub c_state: String,
    // pub c_zip: String,
    // pub c_phone: String,
    // pub c_middle: String,
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

// statement!(
//     insert,
//     history,
//     "   INSERT INTO history (h_c_d_id, h_c_w_id, h_c_id, h_d_id, h_w_id, h_date, h_amount, h_data)
//         VALUES ($1, $2, $3, $4, $5, now()::timestamp, $7, $8);"
// );
