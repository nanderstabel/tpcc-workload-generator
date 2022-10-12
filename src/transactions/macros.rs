#[macro_export]
macro_rules! statement {
    ($command:ident, $name:ident, $query:literal) => {
        paste! {
            fn [<$command _ $name>](
                transaction: &mut Transaction,
                params: &[&(dyn ToSql + Sync)],
            ) -> Result<u64, Error> {
                match stringify!($command) {
                    "delete" | "insert" | "update" => {
                        let res = transaction.execute($query, params)?;
                        let dbg = format!("{}_{}: {}", stringify!($command), stringify!($name), &res);
                        dbg!(dbg);
                        Ok(res)
                    },
                    _ => panic!()
                }
            }
        }
    };
    (read, $name:ident, $query:literal, $result:ty, $column:literal) => {
        paste! {
        fn [<read_ $name>](
            transaction: &mut Transaction,
            params: &[&(dyn ToSql + Sync)],
        ) -> Result<$result, Error> {
                let row = transaction.query_one($query, params)?;
                let res = row.get($column);
                let dbg = format!("read_{}: {}", stringify!($name), &res);
                dbg!(dbg);
                Ok(res)
            }
        }
    };
}
