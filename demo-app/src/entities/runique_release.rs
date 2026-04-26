use runique::prelude::*;

model! {
    RuniqueRelease,
    table: "runique_release",
    pk: id => Pk,
    {
        version:    text [required],
        github_url: url [required],
        crates_url: url [required],
    }
}
