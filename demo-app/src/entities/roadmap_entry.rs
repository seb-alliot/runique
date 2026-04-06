use runique::prelude::*;

model! {
    RoadmapEntry,
    table: "roadmap_entry",
    pk: id => Pk,
    enums: {
        RoadmapStatus: pg [Active, Planned, Future],
    },
    fields: {
        status: enum(RoadmapStatus) [required],
        title: String [required],
        description: String [required],
        link_url: String [nullable],
        link_label: String [nullable],
        link_url_2: String [nullable],
        link_label_2: String [nullable],
        sort_order: i32 [required],
    }
}
