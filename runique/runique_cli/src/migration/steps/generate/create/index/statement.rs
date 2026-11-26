/// The `create_index` statement itself, without any check around it.
pub(super) fn statement(table: &str, idx_name: &str, columns: &[String], unique: bool) -> String {
    let mut cols_chain = String::new();
    for c in columns {
        cols_chain.push_str(&format!(
            "                    .col(Alias::new(\"{c}\"))\n",
            c = c
        ));
    }

    let uniq_line = if unique {
        "                    .unique()\n"
    } else {
        ""
    };

    format!(
        "        manager\n            .create_index(\n                Index::create()\n                    .name(\"{idx}\")\n                    .table(Alias::new(\"{table}\"))\n{cols}{uniq}                    .to_owned(),\n            )\n            .await?;\n",
        idx = idx_name,
        table = table,
        cols = cols_chain,
        uniq = uniq_line
    )
}
