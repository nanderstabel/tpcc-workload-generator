use postgres::types::ToSql;
use postgres::{Error, Transaction};

use crate::statement;
use paste::paste;

pub struct Delivery {
    district_id: i32,
    warehouse_id: i32,
    carrier_id: i32,
}

impl Delivery {
    pub fn new(district_id: i32, warehouse_id: i32, carrier_id: i32) -> Self {
        Self {
            district_id,
            warehouse_id,
            carrier_id,
        }
    }

    pub fn execute(&mut self, mut transaction: Transaction) -> Result<(), Error> {
        let district_id = self.district_id;
        let warehouse_id = self.warehouse_id;
        let carrier_id = self.carrier_id;

        let order_id = read_order_id(&mut transaction, &[&district_id, &warehouse_id])?;
        let customer_id =
            read_customer_id(&mut transaction, &[&order_id, &district_id, &warehouse_id])?;
        update_carrier(
            &mut transaction,
            &[&carrier_id, &order_id, &district_id, &warehouse_id],
        )?;

        update_delivery_date(&mut transaction, &[&order_id, &district_id, &warehouse_id])?;
        let orderline_total =
            read_orderline_total(&mut transaction, &[&order_id, &district_id, &warehouse_id])?;
        update_balance_and_delivery(
            &mut transaction,
            &[&orderline_total, &warehouse_id, &district_id, &customer_id],
        )?;
        delete_neworder(&mut transaction, &[&order_id, &district_id, &warehouse_id])?;

        transaction.commit()
    }
}

statement!(
    read,
    order_id,
    "   SELECT
            no_o_id
        FROM neworder
        WHERE no_d_id = $1
            AND no_w_id = $2
        ORDER BY no_o_id ASC
        LIMIT 1;",
    i32,
    "no_o_id"
);

statement!(
    read,
    customer_id,
    "   SELECT
            o_c_id
        FROM orders
        WHERE o_id = $1
            AND o_d_id = $2
            AND o_w_id = $3;",
    i32,
    "o_c_id"
);

statement!(
    update,
    carrier,
    "   UPDATE orders
        SET o_carrier_id = $1
        WHERE o_id = $2
            AND o_d_id = $3
            AND o_w_id = $4;"
);

statement!(
    update,
    delivery_date,
    "   UPDATE orderline
        SET ol_delivery_d = now()::timestamp
        WHERE ol_o_id = $1
            AND ol_d_id = $2
            AND ol_w_id = $3;"
);

statement!(
    read,
    orderline_total,
    "   SELECT
            sum(ol_amount) AS ol_total
        FROM orderline
        WHERE ol_o_id = $1
            AND ol_d_id = $2
            AND ol_w_id = $3;",
    f32,
    "ol_total"
);

statement!(
    update,
    balance_and_delivery,
    "   UPDATE customer
        SET c_balance = c_balance + $1, c_delivery_cnt = c_delivery_cnt + 1
        WHERE c_w_id = $2
            AND c_d_id = $3
            AND c_id = $4;"
);

statement!(
    delete,
    neworder,
    "   DELETE FROM neworder
        WHERE no_o_id = $1
            AND no_d_id = $2
            AND no_w_id = $3;"
);
