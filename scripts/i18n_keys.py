#!/usr/bin/env python3
"""Réécrit les constantes des clés de traduction à partir des fichiers anglais.

    python3 scripts/i18n_keys.py

Lit `runique/runique_i18n/src/locales/en/` et réécrit
`runique/runique_i18n/src/keys/` : un fichier par sujet, un module par section,
une constante par clé, et la liste `ALL`. À lancer après avoir ajouté, retiré
ou renommé une clé dans les fichiers de traduction, puis `cargo fmt --all`.

Les fichiers anglais font foi : le script ne lit jamais le Rust qu'il a écrit.
Le test `key_parity` vérifie ensuite que les neuf langues ont les mêmes clés.
"""
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
I18N = ROOT / "runique" / "runique_i18n" / "src"
ENGLISH = I18N / "locales" / "en"
KEYS = I18N / "keys"
# La fiche de la langue n'est pas un sujet : elle n'a pas de clé.
NOT_A_SUBJECT = {"lang.json"}


def flatten(node, prefix):
    for name, value in node.items():
        key = f"{prefix}.{name}"
        if isinstance(value, dict):
            yield from flatten(value, key)
        elif isinstance(value, str):
            yield key
        else:
            sys.exit(f"{key} : ni un texte ni une section")


def const_name(leaf):
    name = re.sub(r"(?<=[a-z0-9])(?=[A-Z])", "_", leaf)
    name = re.sub(r"[^A-Za-z0-9]", "_", name).upper()
    return f"CODE_{name}" if name[0].isdigit() else name


def emit(node, indent):
    out = ""
    for name, value in node.items():
        if isinstance(value, str):
            out += f'{indent}pub const {name}: &str = "{value}";\n'
    for name, value in node.items():
        if isinstance(value, dict):
            out += f"{indent}pub mod {name} {{\n{emit(value, indent + '    ')}{indent}}}\n"
    return out


def main():
    keys = []
    for path in sorted(ENGLISH.rglob("*.json")):
        relative = path.relative_to(ENGLISH)
        if relative.as_posix() in NOT_A_SUBJECT:
            continue
        dotted = ".".join(relative.with_suffix("").parts)
        keys += flatten(json.loads(path.read_text(encoding="utf-8")), dotted)
    keys.sort()

    modules = {}
    for key in keys:
        *path, leaf = key.split(".")
        for outer, inner in zip(path, path[1:]):
            if outer == inner:
                sys.exit(f"{key} : un module `{inner}` dans `{outer}` est refusé par clippy")
        node = modules
        for part in path:
            node = node.setdefault(part, {})
        name = const_name(leaf)
        if name in node:
            sys.exit(f"{key} : deux clés donnent la constante {name}")
        node[name] = key

    KEYS.mkdir(exist_ok=True)
    wanted = {f"{subject}.rs" for subject in modules} | {"mod.rs"}
    for stale in KEYS.glob("*.rs"):
        if stale.name not in wanted:
            sys.exit(f"{stale} n'a plus de sujet : retirez-le (gio trash), puis relancez")
    for subject, node in modules.items():
        (KEYS / f"{subject}.rs").write_text(f"//! Keys of `{subject}`.\n" + emit(node, ""), encoding="utf-8")

    header = (
        "//! The key of every message of the framework, as a constant: a key misspelt\n"
        "//! in the code no longer compiles, and renaming one is done here.\n"
        "//!\n"
        "//! One module per subject, the file of the same name in each language\n"
        "//! folder (`locales/<language>/<subject>.json`), then one per section of that\n"
        "//! file. Written from the English files by `scripts/i18n_keys.py`; the\n"
        "//! `key_parity` test holds both lists to each other.\n"
    )
    mods = "".join(f"pub mod {subject};\n" for subject in sorted(modules))
    every = "".join(
        "    self::" + "::".join(key.split(".")[:-1]) + "::" + const_name(key.split(".")[-1]) + ",\n"
        for key in keys
    )
    (KEYS / "mod.rs").write_text(
        f"{header}{mods}\n/// Every key, sorted.\npub const ALL: &[&str] = &[\n{every}];\n", encoding="utf-8"
    )
    print(f"{len(keys)} clés, {len(modules)} sujets : {KEYS.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
