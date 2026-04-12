use runique::prelude::*;

model! {
    RoadmapEntry,
    table: "roadmap_entry",
    pk: id => Pk,
    enums: {
        RoadmapStatus: [Active, Planned, Future],
    },
    {
        status:       choice [enum(RoadmapStatus), required],
        title:        text [required],
        description:  richtext [rows: 5, required],
        link_url:     url,
        link_label:   text,
        link_url_2:   url,
        link_label_2: text,
        sort_order:   int [required],
    }
}
