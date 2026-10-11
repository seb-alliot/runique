//! The list page of one model.
use sea_orm::{EntityTrait, IdenStatic, Order};

/// Only is declared here is shown or accepted from the URL
/// (e.g. `?order=field1,field2`), otherwise it is ignored.
pub struct ListConfig<E: EntityTrait> {
    /// Column shown, in this order.
    display: Vec<E::Column>,
    /// Column linking to the detail page (empty: the first shown)}
    links: Vec<E::Column>,
    /// The only columns sortable from the URL.
    sortable: Vec<E::Column>,
    /// The default order, direction included.
    ordering: Vec<(E::Column, Order)>,
    /// `None: The text column shown, `Some(vec![])`: search disabled.
    search: Option<Vec<E::Column>>,
    /// The only filters accepted from the URL (empty: no filter, `vec![]`: all filters accepted).
    filter: Vec<E::Column>,
    /// The default number of items per page.
    per_page: u64,
}

impl<E: EntityTrait> Default for ListConfig<E> {
    fn default() -> Self {
        Self {
            display: Vec::new(),
            links: Vec::new(),
            sortable: Vec::new(),
            ordering: Vec::new(),
            search: None,
            filter: Vec::new(),
            per_page: 25,
        }
    }
}

impl<E: EntityTrait> ListConfig<E> {
    pub fn display(mut self, cols: impl IntoIterator<Item = E::Column>) -> Self {
        self.display = cols.into_iter().collect();
        self
    }

    pub fn links(mut self, cols: impl IntoIterator<Item = E::Column>) -> Self {
        self.links = cols.into_iter().collect();
        self
    }

    pub fn sortable(mut self, cols: impl IntoIterator<Item = E::Column>) -> Self {
        self.sortable = cols.into_iter().collect();
        self
    }

    pub fn ordering(mut self, cols: impl IntoIterator<Item = (E::Column, Order)>) -> Self {
        self.ordering = cols.into_iter().collect();
        self
    }

    pub fn search(mut self, cols: impl IntoIterator<Item = E::Column>) -> Self {
        self.search = Some(cols.into_iter().collect());
        self
    }

    pub fn filter(mut self, cols: impl IntoIterator<Item = E::Column>) -> Self {
        self.filter = cols.into_iter().collect();
        self
    }

    pub fn per_page(mut self, per_page: u64) -> Self {
        self.per_page = per_page;
        self
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        let mut list_error = Vec::new();
        for (field, cols) in [("links", &self.links), ("sortable", &self.sortable)] {
            for col in cols {
                if !self.display.iter().any(|c| c.as_str() == col.as_str()) {
                    list_error.push(format!(
                        "{field}: column `{}` is not in display",
                        col.as_str()
                    ));
                }
            }
        }
        if !list_error.is_empty() {
            Err(list_error.join("\n"))
        } else {
            Ok(())
        }
    }
}

#[cfg(feature = "sqlite")]
#[cfg(test)]
mod tests {
    use super::ListConfig;
    use crate::admin::refonte_admin::gate::test_support::fixtures::note::{Column, Entity};

    #[test]
    fn links_and_sortable_inside_display_are_accepted() {
        let list = ListConfig::<Entity>::default()
            .display([Column::Id, Column::OwnerId])
            .links([Column::Id])
            .sortable([Column::OwnerId]);
        assert!(list.validate().is_ok());
    }

    #[test]
    fn a_link_outside_display_is_refused() {
        let list = ListConfig::<Entity>::default()
            .display([Column::Id])
            .links([Column::OwnerId]);
        let error = list.validate().unwrap_err();
        assert!(error.contains("links"), "{error}");
        assert!(error.contains("owner_id"), "{error}");
    }

    #[test]
    fn a_sortable_column_outside_display_is_refused() {
        let list = ListConfig::<Entity>::default()
            .display([Column::Id])
            .sortable([Column::OwnerId]);
        let error = list.validate().unwrap_err();
        assert!(error.contains("sortable"), "{error}");
    }
}
