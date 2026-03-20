use runique::prelude::*;

model! {
    RoadmapEntry,
    table: "roadmap_entry",
    pk: id => i32,
    fields: {
        status: String [required],
        title: String [required],
        description: String [required],
        link_url: String [nullable],
        link_label: String [nullable],
        sort_order: i32 [required],
    }
}
