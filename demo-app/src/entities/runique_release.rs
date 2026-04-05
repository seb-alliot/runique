use runique::prelude::*;

model! {
    RuniqueRelease,
    table: "runique_release",
    pk: id => Pk,
    fields: {
        version: String [required],
        github_url: String [required],
        crates_url: String [required],
    }
}
