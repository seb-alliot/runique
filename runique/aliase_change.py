#!/usr/bin/env python3
"""
Script de refactoring automatique : Remplacement des collections par des aliases
Usage: python aliase_change.py runique/src [--dry-run]
"""

import re
import sys
from pathlib import Path
from typing import Dict, List, Tuple
from collections import defaultdict

# =============================================================================
# CONFIGURATION DES REMPLACEMENTS
# =============================================================================

REPLACEMENTS = [
    # HashMap<String, String> → StrMap
    {
        "pattern": r"\bHashMap<String,\s*String>",
        "replacement": "StrMap",
        "import": "use crate::utils::aliases::StrMap;",
        "description": "HashMap<String, String> → StrMap",
    },
    # HashMap<String, Vec<String>> → StrVecMap
    {
        "pattern": r"\bHashMap<String,\s*Vec<String>>",
        "replacement": "StrVecMap",
        "import": "use crate::utils::aliases::StrVecMap;",
        "description": "HashMap<String, Vec<String>> → StrVecMap",
    },
    # HashMap<String, Value> → JsonMap
    {
        "pattern": r"\bHashMap<String,\s*Value>",
        "replacement": "JsonMap",
        "import": "use crate::utils::aliases::JsonMap;",
        "description": "HashMap<String, Value> → JsonMap",
    },
    # IndexMap<String, Box<dyn FormField>> → FieldsMap
    {
        "pattern": r"\bIndexMap<String,\s*Box<dyn\s+FormField>>",
        "replacement": "FieldsMap",
        "import": "use crate::utils::aliases::FieldsMap;",
        "description": "IndexMap<String, Box<dyn FormField>> → FieldsMap",
    },
    # Vec<FlashMessage> → Messages
    {
        "pattern": r"\bVec<FlashMessage>",
        "replacement": "Messages",
        "import": "use crate::utils::aliases::Messages;",
        "description": "Vec<FlashMessage> → Messages",
    },
]

# Fichiers à ignorer
IGNORE_FILES = [
    "src/utils/aliases/definition.rs",  # Fichier de définition des aliases
    "src/lib.rs",                 # Géré manuellement
]

# =============================================================================
# FONCTIONS UTILITAIRES
# =============================================================================

def find_rust_files(root_path: Path) -> List[Path]:
    """Trouve tous les fichiers .rs récursivement."""
    rust_files = []
    for file_path in root_path.rglob("*.rs"):
        # Ignore les fichiers de test et build
        if "target" in file_path.parts or "tests" in file_path.parts:
            continue
        # Ignore les fichiers spécifiques
        if any(ignore in str(file_path) for ignore in IGNORE_FILES):
            continue
        rust_files.append(file_path)
    return sorted(rust_files)


def has_existing_import(content: str, import_line: str) -> bool:
    """Vérifie si l'import existe déjà."""
    # Nettoie l'import pour la comparaison
    clean_import = import_line.replace("use crate::utils::aliases::", "").rstrip(";")

    # Cherche les patterns d'import possibles
    patterns = [
        rf"use crate::utils::aliases::{re.escape(clean_import)};",
        rf"use crate::utils::aliases::\{{[^}}]*{re.escape(clean_import)}[^}}]*\}};",
        rf"use crate::prelude::\*;",  # Import wildcard du prelude
    ]

    for pattern in patterns:
        if re.search(pattern, content):
            return True
    return False


def add_import_to_file(content: str, import_line: str) -> str:
    """Ajoute l'import au bon endroit dans le fichier."""
    # Si l'import existe déjà, ne rien faire
    if has_existing_import(content, import_line):
        return content

    lines = content.split('\n')

    # Trouve la dernière ligne d'import use
    last_use_index = -1
    for i, line in enumerate(lines):
        if line.strip().startswith("use "):
            last_use_index = i

    # Insère après le dernier use trouvé
    if last_use_index >= 0:
        lines.insert(last_use_index + 1, import_line)
    else:
        # Sinon, insère après les commentaires d'en-tête
        insert_index = 0
        for i, line in enumerate(lines):
            if not line.strip().startswith("//") and line.strip():
                insert_index = i
                break
        lines.insert(insert_index, import_line)
        lines.insert(insert_index + 1, "")  # Ligne vide

    return '\n'.join(lines)


def refactor_file(file_path: Path, dry_run: bool = False) -> Dict:
    """Refactorise un fichier Rust."""
    result = {
        "path": str(file_path),
        "changes": [],
        "imports_added": [],
        "modified": False,
    }

    # Lit le contenu
    try:
        content = file_path.read_text(encoding='utf-8')
    except Exception as e:
        result["error"] = str(e)
        return result

    original_content = content
    imports_to_add = set()

    # Applique chaque remplacement
    for replacement in REPLACEMENTS:
        pattern = replacement["pattern"]
        new_name = replacement["replacement"]
        import_line = replacement["import"]

        # Compte les occurrences
        matches = re.findall(pattern, content)
        if matches:
            # Remplace
            content = re.sub(pattern, new_name, content)

            # Enregistre le changement
            result["changes"].append({
                "description": replacement["description"],
                "count": len(matches),
            })

            # Marque l'import à ajouter
            imports_to_add.add(import_line)

    # Ajoute les imports nécessaires
    for import_line in sorted(imports_to_add):
        if not has_existing_import(content, import_line):
            content = add_import_to_file(content, import_line)
            result["imports_added"].append(import_line)

    # Vérifie si le fichier a été modifié
    if content != original_content:
        result["modified"] = True

        # Écrit le fichier (sauf en dry-run)
        if not dry_run:
            try:
                file_path.write_text(content, encoding='utf-8')
            except Exception as e:
                result["error"] = str(e)

    return result


def print_report(results: List[Dict], dry_run: bool):
    """Affiche le rapport des changements."""
    total_files = len(results)
    modified_files = sum(1 for r in results if r["modified"])
    total_changes = sum(
        sum(c["count"] for c in r["changes"])
        for r in results if "changes" in r
    )

    print("\n" + "=" * 80)
    print(" RAPPORT DE REFACTORING")
    print("=" * 80)

    if dry_run:
        print(" MODE DRY-RUN (aucun fichier n'a été modifié)")
    else:
        print(" MODIFICATIONS APPLIQUÉES")

    print(f"\n Fichiers scannés: {total_files}")
    print(f"  Fichiers modifiés: {modified_files}")
    print(f" Total de remplacements: {total_changes}")

    # Détails par type de changement
    change_stats = defaultdict(int)
    for result in results:
        for change in result.get("changes", []):
            change_stats[change["description"]] += change["count"]

    if change_stats:
        print("\n Détails des remplacements:")
        for desc, count in sorted(change_stats.items()):
            print(f"   • {desc}: {count} occurrences")

    # Liste les fichiers modifiés
    if modified_files > 0:
        print("\n Fichiers modifiés:")
        for result in results:
            if result["modified"]:
                file_path = result["path"]
                changes_count = sum(c["count"] for c in result["changes"])
                imports_count = len(result.get("imports_added", []))

                print(f"\n   {file_path}")
                print(f"      → {changes_count} remplacement(s)")
                if imports_count > 0:
                    print(f"      → {imports_count} import(s) ajouté(s)")

                # Détails des changements
                for change in result["changes"]:
                    print(f"         • {change['description']}: {change['count']}×")

    # Erreurs éventuelles
    errors = [r for r in results if "error" in r]
    if errors:
        print("\n ERREURS:")
        for result in errors:
            print(f"   • {result['path']}: {result['error']}")

    print("\n" + "=" * 80)

    if dry_run:
        print("\n Pour appliquer les changements, relancez sans --dry-run")
    else:
        print("\n Refactoring terminé avec succès!")
        print(" N'oubliez pas de lancer: cargo check && cargo test")


# =============================================================================
# MAIN
# =============================================================================

def main():
    # Parse arguments
    if len(sys.argv) < 2:
        print("Usage: python refactor_collections.py /chemin/vers/runique/src [--dry-run]")
        sys.exit(1)

    root_path = Path(sys.argv[1])
    dry_run = "--dry-run" in sys.argv

    if not root_path.exists():
        print(f"Erreur: Le chemin '{root_path}' n'existe pas")
        sys.exit(1)

    print(f"🔍 Scan de {root_path}...")

    # Trouve tous les fichiers Rust
    rust_files = find_rust_files(root_path)
    print(f"📁 {len(rust_files)} fichiers .rs trouvés")

    if dry_run:
        print("🔍 Mode DRY-RUN activé (simulation)")

    # Refactorise chaque fichier
    results = []
    for i, file_path in enumerate(rust_files, 1):
        print(f"[{i}/{len(rust_files)}] {file_path.name}...", end='\r')
        result = refactor_file(file_path, dry_run=dry_run)
        results.append(result)

    print(" " * 80, end='\r')  # Clear progress line

    # Affiche le rapport
    print_report(results, dry_run)


if __name__ == "__main__":
    main()