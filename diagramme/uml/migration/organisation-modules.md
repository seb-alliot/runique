# Migration — organisation des modules (2026-11-12)

Le dossier `migration/` suit les étapes que traverse `makemigrations`. `cli/` ne contient que les
appels des commandes ; la logique vit dans son domaine.

```mermaid
flowchart LR
    BIN["bin/runique.rs"] --> CLI["cli/<br/>makemigration · migrate · new_project · create_superuser<br/>(appels seulement)"]
    CLI --> RUN["migration/cli_migration<br/>run · migrate · init_migrate"]
    CLI --> NP["new_project/<br/>runique new"]
    CLI --> SU["auth/create_superuser"]
    NP --> INIT["cli_migration/init_migrate<br/>crate migration/"]

    subgraph pipeline["makemigrations (migration/)"]
        direction LR
        SCAN["cli_migration/scan<br/>+ snapshot"] --> PARSE["① parse/<br/>builder · extend · seaorm<br/>→ ParsedSchema"]
        PARSE --> DIFF["② diff.rs<br/>→ Changes"]
        DIFF --> CHG["③ changes/<br/>ordering · destructive_changes<br/>· name_length_limit"]
        CHG --> REP["cli_migration/report<br/>refus sans --force"]
        CHG --> GEN["④ generate/<br/>create · alter · columns<br/>· constraints · enums"]
        GEN --> PLAN["cli_migration/plan<br/>écriture atomique + lib_file"]
    end
    RUN --> SCAN

    SCHEMA["schema/<br/>ModelSchema · ColumnDef · FK · index · hooks<br/>(utilisé à l'exécution par model!{})"]
```

| Dossier | Pur ? | Rôle |
|---|---|---|
| `schema/` | oui | décrire un modèle (code généré par `model!{}`, formulaires) |
| `parse/` | oui | lire le DSL, `extend!{}` et les fichiers SeaORM existants |
| `diff.rs` | oui | comparer le snapshot et le modèle |
| `changes/` | oui | trier par FK, lister les changements destructifs et les noms trop longs |
| `generate/` | oui | produire le texte des fichiers de migration |
| `cli_migration/` | non (fichiers, terminal) | scanner, lire les snapshots, refuser, écrire, `migration up/down/status`, crate `migration/` |

Règle : `cli_migration/` appelle les dossiers purs, jamais l'inverse ; `cli/` n'appelle que
`cli_migration/`, `new_project/` et `auth/`.
