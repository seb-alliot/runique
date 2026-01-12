use crate::formulaire::field::SelectOption;

/// Convertit un Vec de tuples (&str, &str) en Vec<SelectOption>
pub fn to_options(opts: Vec<(&str, &str)>) -> Vec<SelectOption> {
    opts.into_iter()
        .map(|(v, l)| SelectOption {
            value: v.to_string(),
            label: l.to_string()
        })
        .collect()
}