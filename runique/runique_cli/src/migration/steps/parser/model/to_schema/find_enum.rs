use super::*;

pub(super) fn find_enum<'a>(enums: &'a [EnumDef], name: &syn::Ident) -> Option<&'a EnumDef> {
    enums.iter().find(|e| e.name == *name)
}
