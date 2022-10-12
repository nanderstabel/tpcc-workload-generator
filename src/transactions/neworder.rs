use postgres::types::ToSql;
use postgres::{Error, Transaction};

use itertools::zip_eq;

use crate::statement;
use paste::paste;

pub struct NewOrder {
    district_id: i32,
    warehouse_id: i32,
    customer_id: i32,
    orderline_count: i32,
    o_all_local: i32,
    item_ids: Vec<i32>,
    orderline_quantities: Vec<i32>,
}

impl NewOrder {
    pub fn new(
        district_id: i32,
        warehouse_id: i32,
        customer_id: i32,
        orderline_count: i32,
        o_all_local: i32,
        item_ids: Vec<i32>,
        orderline_quantities: Vec<i32>,
    ) -> Self {
        Self {
            district_id,
            warehouse_id,
            customer_id,
            orderline_count,
            o_all_local,
            item_ids,
            orderline_quantities,
        }
    }

    pub fn execute(&self, mut transaction: Transaction) -> Result<(), Error> {
        let district_id = self.district_id;
        let warehouse_id = self.warehouse_id;
        let customer_id = self.customer_id;
        let orderline_count = self.orderline_count;
        let o_all_local = self.o_all_local;

        let next_order_id = read_next_order_id(&mut transaction, &[&warehouse_id, &district_id])?;

        insert_neworder(
            &mut transaction,
            &[&next_order_id, &warehouse_id, &district_id],
        )?;

        update_district(&mut transaction, &[&warehouse_id, &district_id])?;

        insert_order(
            &mut transaction,
            &[
                &next_order_id,
                &district_id,
                &warehouse_id,
                &customer_id,
                &orderline_count,
                &o_all_local,
            ],
        )?;

        let supply_warehouse_id: i32 = 1;

        for (orderline_number, (item_id, orderline_quantity)) in
            zip_eq(self.item_ids.iter(), self.orderline_quantities.iter()).enumerate()
        {
            let item_price = read_item_price(&mut transaction, &[&item_id])?;

            let orderline_amount = *orderline_quantity as f32 * item_price;

            let (stock_quantity, district_info) = read_quantity_and_district_info(
                &mut transaction,
                &[&item_id, &warehouse_id],
                district_id,
            )?;

            insert_orderline(
                &mut transaction,
                &[
                    &next_order_id,
                    &district_id,
                    &warehouse_id,
                    &(orderline_number as i32),
                    &item_id,
                    &supply_warehouse_id,
                    orderline_quantity,
                    &orderline_amount,
                    &district_info,
                ],
            )?;

            let remote_cnt_increment: i32 = 0;
            update_stock(
                &mut transaction,
                &[
                    &stock_quantity,
                    &remote_cnt_increment,
                    &item_id,
                    &supply_warehouse_id,
                ],
            )?;
        }

        transaction.commit()
    }
}

statement!(
    read,
    next_order_id,
    "   SELECT
            d_next_o_id,
            d_tax
        FROM district
        WHERE d_w_id = $1
            AND d_id = $2
        FOR UPDATE;",
    i32,
    "d_next_o_id"
);

statement!(
    insert,
    neworder,
    "   INSERT INTO neworder (no_o_id, no_d_id, no_w_id)
        VALUES ($1, $2, $3);"
);

statement!(
    update,
    district,
    "   UPDATE district
        SET d_next_o_id = d_next_o_id + 1
        WHERE d_w_id = $1
            AND d_id = $2;"
);

statement!(
    insert,
    order,
    "   INSERT INTO orders (o_id, o_d_id, o_w_id, o_c_id, o_entry_d, o_ol_cnt, o_all_local)
        VALUES ($1, $2, $3, $4, now()::timestamp, $5, $6);"
);

statement!(
    read,
    item_price,
    "   SELECT
            i_price,
            i_name,
            i_data
        FROM item
        WHERE i_id = $1;",
    f32,
    "i_price"
);

fn read_quantity_and_district_info(
    transaction: &mut Transaction,
    params: &[&(dyn ToSql + Sync)],
    district_id: i32,
) -> Result<(i32, String), Error> {
    let row = transaction.query_one(
        "
        SELECT
            st_quantity,
            st_dist_01,
            st_dist_02,
            st_dist_03,
            st_dist_04,
            st_dist_05,
            st_dist_06,
            st_dist_07,
            st_dist_08,
            st_dist_09,
            st_dist_10
        FROM stock
        WHERE st_i_id = $1
            AND st_w_id = $2;",
        params,
    )?;

    Ok((
        row.get("st_quantity"),
        row.get(format!("st_dist_{:02}", district_id).as_str()),
    ))
}

statement!(
    insert,
    orderline,
    "   INSERT INTO orderline (ol_o_id, ol_d_id, ol_w_id, ol_number, ol_i_id, ol_supply_w_id, ol_quantity, ol_amount, ol_dist_info)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9);"
);

statement!(
    update,
    stock,
    "   UPDATE stock
        SET st_quantity = $1,
            st_ytd = st_ytd + $2,
            st_order_cnt = st_order_cnt + 1,
            st_remote_cnt = st_remote_cnt + 1
        WHERE st_i_id = $3
            AND st_w_id = $4;"
);
