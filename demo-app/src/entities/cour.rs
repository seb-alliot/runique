use runique::prelude::*;

model! {
    Cour,
    table: "cour",
    pk: id => Pk,
    enums: {
        CourTheme: [
            Fondamentaux = "Fondamentaux",
            MemoireSurete = "Mémoire & sûreté",
            Indispensables = "Indispensables",
            Avance = "Avancé",
            Runique = "Runique"
        ],
        Difficulte: [
            Debutant = "debutant",
            Intermediaire = "intermediaire",
            Avance = "avance",
            Specifique = "specifique"
        ],
    },
    {
        slug:       text [required],
        lang:       text [required],
        title:      text [required],
        theme:      choice [enum(CourTheme), required],
        difficulte: choice [enum(Difficulte), required],
        ordre:      int [required],
        sort_order: int [required],
    }
}
