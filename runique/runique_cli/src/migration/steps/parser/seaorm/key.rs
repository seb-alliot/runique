use super::*;

pub(super) fn extract_seaorm_fk(expr: &Expr) -> Option<ParsedFk> {
    let chain = collect_chain(expr);
    let mut from_col: Option<String> = None;
    let mut to_table: Option<String> = None;
    let mut to_col: Option<String> = None;
    let mut on_delete = FkAction::NoAction;
    let mut on_update = FkAction::NoAction;

    for mc in &chain {
        match mc.method.to_string().as_str() {
            "from" if mc.args.len() >= 2 => {
                from_col = extract_alias_new_str_inner(&mc.args[1]);
            }
            "to" if mc.args.len() >= 2 => {
                to_table = extract_alias_new_str_inner(&mc.args[0]);
                to_col = extract_alias_new_str_inner(&mc.args[1]);
            }
            "on_delete" => {
                if let Some(arg) = mc.args.first() {
                    on_delete = extract_fk_action_value(arg);
                }
            }
            "on_update" => {
                if let Some(arg) = mc.args.first() {
                    on_update = extract_fk_action_value(arg);
                }
            }
            _ => {}
        }
    }

    Some(ParsedFk {
        from_column: from_col?,
        to_table: to_table?,
        to_column: to_col.unwrap_or_else(|| "id".to_string()),
        on_delete,
        on_update,
    })
}
