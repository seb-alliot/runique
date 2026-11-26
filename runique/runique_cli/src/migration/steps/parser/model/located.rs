/// `line:column: message` — where a DSL error is in the scanned file.
pub(crate) fn located(err: &syn::Error) -> String {
    let start = err.span().start();
    format!("{}:{}: {}", start.line, start.column + 1, err)
}
