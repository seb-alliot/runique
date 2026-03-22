#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""
Parse les fichiers markdown de docs/fr/cour/ et génère les INSERT SQL
pour les tables chapitre et doc_block (via chapitre_id).

Mapping :
  fichier .md       → cour    (slug = nom du fichier)
  ## Titre section  → chapitre
  contenu entre ##  → doc_block (types : text, code, list, table, warning)

Usage :
  uv run docs/fr/md_to_seed.py
  → génère docs/fr/seed_cours.sql
"""

import re
from pathlib import Path

COUR_DIR = Path(__file__).parent.parent / "docs" / "fr" / "cour"
OUT_FILE = Path(__file__).parent / "seed_cours.sql"

# Correspondance slug fichier → id dans la table cour (doit matcher le seed)
COUR_IDS = {
    "cargo-dependances":        1,
    "variables-et-fonctions":   2,
    "structures-et-controle":   3,
    "structures-enums":    4,
    "pattern-matching":    5,
    "collections":         6,
    "modules":             7,
    "tests-rust":          8,
    "closures-iterateurs": 9,
    "gestion-des-erreurs": 10,
    "generics":            11,
    "type-aliases":        12,
    "lifetimes":           13,
    "box-dynamiques":      14,
    "smart-pointers":      15,
    "send-sync":           16,
    "traits-avances":      17,
    "macros-declaratives":    18,
    "macros-export":          23,
    "macros-derive":          24,
    "macros-attribut":        25,
    "macros-function-like":   26,
    "async-tokio":            19,
    "orm":                 20,
    "cours-filtre-admin":  21,
    "middleware-ordre":    22,
}


def slugify(titre: str) -> str:
    s = titre.lower()
    replacements = {
        "é": "e", "è": "e", "ê": "e", "ë": "e",
        "à": "a", "â": "a", "ä": "a",
        "î": "i", "ï": "i", "ô": "o",
        "ù": "u", "û": "u", "ü": "u", "ç": "c",
        "<": "", ">": "", "`": "", "(": "", ")": "",
        "!": "", "?": "", ",": "", ".": "", ":": "",
        "&": "et",
    }
    for src, dst in replacements.items():
        s = s.replace(src, dst)
    s = re.sub(r"[\s_/]+", "-", s)
    s = re.sub(r"[^a-z0-9\-]", "", s)
    s = re.sub(r"-+", "-", s).strip("-")
    return s[:80]


def detect_block_type(content: str) -> str:
    stripped = content.strip()
    if stripped.startswith("```"):
        return "code"
    if re.match(r"^\|.+\|", stripped):
        return "table"
    if re.match(r"^[-*]\s", stripped) or re.match(r"^\d+\.\s", stripped):
        return "list"
    if re.match(r"^>\s+\*\*(important|attention|warning|note)", stripped, re.I):
        return "warning"
    if stripped.startswith("> "):
        return "warning"
    return "text"


def escape_pg(s: str) -> str:
    """Échappe pour PostgreSQL dollar-quoting fallback via quotes simples."""
    return s.replace("'", "''")


def split_into_chapitres(contenu: str) -> list[dict]:
    """
    Découpe le markdown en chapitres délimités par les ## de niveau 2.
    Retourne une liste de {title, blocks_raw}.
    """
    chapitres = []
    current_title = None
    current_lines = []

    for line in contenu.splitlines():
        # Détecte ## Titre (niveau 2 uniquement)
        m = re.match(r"^##\s+(.+)$", line)
        if m:
            if current_title is not None:
                chapitres.append({
                    "title": current_title,
                    "raw": "\n".join(current_lines).strip(),
                })
            current_title = m.group(1).strip()
            current_lines = []
        else:
            if current_title is not None:
                current_lines.append(line)

    if current_title and current_lines:
        chapitres.append({
            "title": current_title,
            "raw": "\n".join(current_lines).strip(),
        })

    # Filtre les chapitres vides ou de type "table des matières"
    filtre = []
    for ch in chapitres:
        titre_low = ch["title"].lower()
        if any(x in titre_low for x in ["table des matières", "objectifs"]):
            continue
        if not ch["raw"].strip():
            continue
        filtre.append(ch)

    return filtre


def split_into_blocks(raw: str) -> list[dict]:
    """
    Découpe le contenu d'un chapitre en blocs (text, code, list, table, warning).
    Les ### sous-titres deviennent un bloc 'text' avec heading.
    """
    blocks = []
    lines = raw.splitlines()
    i = 0

    while i < len(lines):
        line = lines[i]

        # Sous-titre ### → heading du prochain bloc
        heading_match = re.match(r"^###\s+(.+)$", line)
        if heading_match:
            heading = heading_match.group(1).strip()
            i += 1
            # Collecte le contenu qui suit jusqu'au prochain ### ou fin
            body_lines = []
            while i < len(lines) and not re.match(r"^###\s+", lines[i]):
                body_lines.append(lines[i])
                i += 1
            body = "\n".join(body_lines).strip()
            if body:
                blocks.append({
                    "heading": heading,
                    "content": body,
                    "type": detect_block_type(body),
                })
            continue

        # Bloc de code ```
        if line.startswith("```"):
            code_lines = [line]
            i += 1
            while i < len(lines) and not lines[i].startswith("```"):
                code_lines.append(lines[i])
                i += 1
            if i < len(lines):
                code_lines.append(lines[i])
                i += 1
            content = "\n".join(code_lines).strip()
            blocks.append({"heading": None, "content": content, "type": "code"})
            continue

        # Ligne vide → passe
        if not line.strip():
            i += 1
            continue

        # Bloc texte / liste / table / warning : collecte jusqu'à ligne vide
        block_lines = []
        while i < len(lines) and lines[i].strip():
            # S'arrête si on tombe sur un ```  ou ###
            if lines[i].startswith("```") or re.match(r"^###\s+", lines[i]):
                break
            block_lines.append(lines[i])
            i += 1

        if block_lines:
            content = "\n".join(block_lines).strip()
            blocks.append({
                "heading": None,
                "content": content,
                "type": detect_block_type(content),
            })

    return [b for b in blocks if b["content"].strip()]


def generer_sql(md_path: Path, cour_id: int, chapitre_id_start: int, block_id_start: int) -> tuple[str, int, int]:
    contenu = md_path.read_text(encoding="utf-8")
    chapitres = split_into_chapitres(contenu)

    if not chapitres:
        return "", chapitre_id_start, block_id_start

    slug_base = md_path.stem
    lines = [f"\n-- {md_path.name} (cour_id={cour_id})"]

    chap_inserts = []
    block_inserts = []

    chap_id = chapitre_id_start
    blk_id = block_id_start

    for sort_order, ch in enumerate(chapitres, start=1):
        chap_slug = f"{slug_base}-{slugify(ch['title'])}"
        title_esc = escape_pg(ch["title"])
        chap_inserts.append(
            f"({chap_id}, {cour_id}, '{chap_slug}', '{title_esc}', NULL, {sort_order})"
        )

        blocks = split_into_blocks(ch["raw"])
        for blk_order, blk in enumerate(blocks, start=1):
            heading_sql = f"'{escape_pg(blk['heading'])}'" if blk["heading"] else "NULL"
            content = blk["content"]
            # Utilise dollar-quoting PostgreSQL
            tag = f"$BLK{blk_id}$"
            content_sql = f"{tag}{content}{tag}"
            block_inserts.append(
                f"({blk_id}, {chap_id}, {heading_sql}, {content_sql}, '{blk['type']}', {blk_order})"
            )
            blk_id += 1

        chap_id += 1

    if chap_inserts:
        lines.append("INSERT INTO chapitre (id, cour_id, slug, title, lead, sort_order) VALUES")
        lines.append(",\n".join(chap_inserts) + ";")

    if block_inserts:
        lines.append("INSERT INTO cour_block (id, chapitre_id, heading, content, block_type, sort_order) VALUES")
        lines.append(",\n".join(block_inserts) + ";")

    return "\n".join(lines), chap_id, blk_id


def main():
    header = """\
\\encoding utf8

-- ============================================================
-- seed_cours.sql — chapitres et blocs générés depuis docs/fr/cour/
-- Idempotent : DELETE + INSERT
-- ============================================================

DELETE FROM cour_block;
DELETE FROM chapitre;
DELETE FROM cour;

INSERT INTO cour (id, slug, lang, title, theme, difficulte, sort_order, ordre) VALUES
( 1, 'cargo-dependances',   'fr', 'Cargo & dépendances',    'Fondamentaux',     'debutant',      1,  1),
( 2, 'variables-fonctions', 'fr', 'Variables & fonctions',  'Fondamentaux',     'debutant',      2,  2),
( 3, 'structures-controle', 'fr', 'Structures & contrôle',  'Fondamentaux',     'debutant',      3,  3),
( 4, 'structures-enums',    'fr', 'Structures & enums',     'Fondamentaux',     'debutant',      4,  4),
( 5, 'pattern-matching',    'fr', 'Pattern matching',       'Fondamentaux',     'debutant',      5,  5),
( 6, 'collections',         'fr', 'Collections',            'Fondamentaux',     'debutant',      6,  6),
( 7, 'modules',             'fr', 'Modules',                'Fondamentaux',     'debutant',      7,  7),
( 8, 'tests-rust',          'fr', 'Tests en Rust',          'Fondamentaux',     'debutant',      8,  8),
( 9, 'closures-iterateurs', 'fr', 'Closures & itérateurs',  'Mémoire & sûreté', 'intermediaire', 1,  9),
(10, 'gestion-erreurs',     'fr', 'Gestion des erreurs',    'Mémoire & sûreté', 'intermediaire', 2, 10),
(11, 'generics',            'fr', 'Generics',               'Mémoire & sûreté', 'intermediaire', 3, 11),
(12, 'type-aliases',        'fr', 'Type aliases',           'Mémoire & sûreté', 'intermediaire', 4, 12),
(13, 'lifetimes',           'fr', 'Lifetimes',              'Mémoire & sûreté', 'intermediaire', 5, 13),
(14, 'box-dynamiques',      'fr', 'Box & types dynamiques', 'Mémoire & sûreté', 'intermediaire', 6, 14),
(15, 'smart-pointers',      'fr', 'Smart pointers',         'Mémoire & sûreté', 'intermediaire', 7, 15),
(16, 'send-sync',           'fr', 'Send & Sync',            'Mémoire & sûreté', 'intermediaire', 8, 16),
(17, 'traits-avances',      'fr', 'Traits avancés',         'Avancé',           'avance',        1, 17),
(18, 'macros-declaratives',  'fr', 'Macros déclaratives',                 'Avancé', 'avance', 2, 18),
(23, 'macros-export',        'fr', 'Macros — Visibilité et export',      'Avancé', 'avance', 3, 23),
(24, 'macros-derive',        'fr', 'Macros procédurales — Derive',       'Avancé', 'avance', 4, 24),
(25, 'macros-attribut',      'fr', 'Macros procédurales — Attribute',    'Avancé', 'avance', 5, 25),
(26, 'macros-function-like', 'fr', 'Macros procédurales — Function-like','Avancé', 'avance', 6, 26),
(19, 'async-tokio',          'fr', 'Async & Tokio',                      'Avancé', 'avance', 7, 19),
(20, 'orm-seaorm',          'fr', 'ORM — SeaORM',           'Runique',          'specifique',    1, 20),
(21, 'filtre-admin',        'fr', 'Filtre admin',           'Runique',          'specifique',    2, 21),
(22, 'middleware-ordre',   'fr', 'Middlewares — Pièges et Solutions', 'Runique', 'specifique',    3, 22);
"""

    footer = """
-- Reset séquences
SELECT setval('chapitre_id_seq', (SELECT MAX(id) FROM chapitre));
"""

    parts = [header]

    chapitre_id = 1
    block_id = 10000  # offset pour ne pas collider avec les blocs doc existants

    for slug, cour_id in sorted(COUR_IDS.items(), key=lambda x: x[1]):
        # Cherche le fichier .md correspondant
        candidates = list(COUR_DIR.glob(f"{slug}*.md")) + list(COUR_DIR.glob(f"*{slug}*.md"))
        candidates = [c for c in candidates if c.stem == slug or c.stem.replace("_", "-") == slug]

        if not candidates:
            print(f"  [skip] aucun fichier pour '{slug}'")
            continue

        md_path = candidates[0]
        sql, chapitre_id, block_id = generer_sql(md_path, cour_id, chapitre_id, block_id)
        if sql:
            parts.append(sql)
            print(f"  OK  {md_path.name}")
        else:
            print(f"  [vide] {md_path.name}")

    parts.append(footer)
    OUT_FILE.write_text("\n".join(parts), encoding="utf-8")
    print(f"\nGenere : {OUT_FILE}")


if __name__ == "__main__":
    main()
