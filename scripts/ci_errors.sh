#!/usr/bin/env bash
# Tout ce qu'un lancement de tests a signalé, extrait de son journal.
#
#   bash scripts/ci_errors.sh                 # lit target/tests.log
#   bash scripts/ci_errors.sh autre.log       # lit un autre journal
#   cargo test 2>&1 | bash scripts/ci_errors.sh -   # lit l'entrée standard
#
# Le journal est celui du bloc « Reproduire la CI en local » (target/tests.log).
# Le script ne lance rien : il ne fait que lire. Les lignes de journal (ERROR,
# WARN) sont regroupées et comptées, heure retirée, pour qu'un même message
# émis cent fois tienne sur une ligne. Une ligne ERROR n'est pas un échec : un
# test qui vérifie une page 404 en écrit une. La première section dit ce qui
# a réellement échoué.
set -u

source="${1:-target/tests.log}"
propre="$(mktemp)"
lu=""
trap 'rm -f "$propre" $lu' EXIT
if [ "$source" = "-" ]; then
    lu="$(mktemp)"
    journal="$lu"
    cat > "$journal"
elif [ -r "$source" ]; then
    journal="$source"
else
    echo "Journal introuvable : $source" >&2
    echo "Lancez d'abord le bloc de la CI, ou passez un fichier : bash scripts/ci_errors.sh <fichier>" >&2
    exit 2
fi

# Sans les couleurs du terminal ni les retours chariot.
sed -E 's/\x1B\[[0-9;]*[A-Za-z]//g; s/\r$//' "$journal" > "$propre"

titre() { printf '\n===== %s =====\n' "$1"; }

# Les lignes qui correspondent à $1, regroupées, les plus fréquentes d'abord.
compte() {
    grep -E "$1" "$propre" | sed -E "${2:-s/^//}" | sort | uniq -c | sort -rn
}

# L'heure en tête d'une ligne de journal, et les nombres qui changent d'un
# lancement à l'autre (durées, adresses) ne distinguent pas deux messages.
SANS_HEURE='s/^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9:.]+Z[[:space:]]+//'

titre "Ce qui a échoué"
echecs="$(grep -cE '^test .* FAILED$|^test result: FAILED|^error(\[E[0-9]+\])?:|panicked at' "$propre")"
if [ "$echecs" -eq 0 ]; then
    echo "Rien : aucun test en échec, aucune erreur de compilation, aucune panique."
else
    grep -nE '^===== |^test .* FAILED$|^test result: FAILED|^error(\[E[0-9]+\])?:|panicked at' "$propre"
fi

titre "Bilan de chaque lot"
if ! grep -qE '^test result:' "$propre"; then
    echo "Aucun résultat de test dans ce journal : les tests n'ont pas encore tourné, ou la compilation est en cours."
fi
grep -E '^===== |^test result:' "$propre" \
    | awk '/^=====/{lot=$0; next} {print lot " -> " $0}' \
    | sed -E 's/^===== (.*) ===== -> test result: /\1 : /' \
    | awk -F' : ' '{n[$1]++; last[$1]=$2; if ($2 ~ /FAILED/) bad[$1]=1}
                   END {for (l in n) printf "%s%s : %d binaire(s) de test, dernier : %s\n", (l in bad ? "ÉCHEC  " : "ok     "), l, n[l], last[l]}' \
    | sort

titre "Erreurs de compilation"
compte '^error(\[E[0-9]+\])?:' || true
[ "$(grep -cE '^error(\[E[0-9]+\])?:' "$propre")" -eq 0 ] && echo "Aucune."

titre "Avertissements du compilateur et de clippy"
compte '^warning: ' 's/^warning: //' | grep -vE 'generated [0-9]+ warnings?|build failed' || echo "Aucun."

titre "Paniques"
compte 'panicked at' 's/^thread .([^ ]*). (\([0-9]+\) )?panicked at /\1 : /' || true
[ "$(grep -cE 'panicked at' "$propre")" -eq 0 ] && echo "Aucune."

titre "Journal : lignes ERROR"
compte "^([0-9]{4}-[0-9-]+T[0-9:.]+Z)?[[:space:]]*ERROR " "$SANS_HEURE" || true
[ "$(grep -cE '^([0-9]{4}-[0-9-]+T[0-9:.]+Z)?[[:space:]]*ERROR ' "$propre")" -eq 0 ] && echo "Aucune."

titre "Journal : lignes WARN"
compte "^([0-9]{4}-[0-9-]+T[0-9:.]+Z)?[[:space:]]*WARN " "$SANS_HEURE" || true
[ "$(grep -cE '^([0-9]{4}-[0-9-]+T[0-9:.]+Z)?[[:space:]]*WARN ' "$propre")" -eq 0 ] && echo "Aucune."

titre "Messages de Runique écrits hors journal"
# Ce que le framework écrit directement sur la sortie d'erreur, sans niveau.
# Les mêmes messages passés par le journal sont déjà comptés plus haut.
hors_journal="$(grep -E 'Runique log configuration|Feature `test-utils`|DEBUG=true' "$propre" | grep -vE ' (WARN|ERROR|INFO|DEBUG|TRACE) ' | sort | uniq -c | sort -rn)"
if [ -n "$hors_journal" ]; then echo "$hors_journal"; else echo "Aucun."; fi

titre "Clés de traduction affichées à la place de leur message"
# Une clé sans message s'affiche telle quelle : `sujet.nature.clé`, ou
# `admin.écran.nature.clé`. Trouvée ici, elle manque dans les fichiers de
# traduction ou son nom est faux dans le code.
cles='(^|[^A-Za-z0-9_.:/"])((admin\.[a-z_]+|[a-z_]+)\.(error|label|hint|log|info)\.[A-Za-z0-9_]+)'
if grep -qE "$cles" "$propre"; then
    grep -oE "$cles" "$propre" | sed -E 's/^[^a-z]*//' | sort | uniq -c | sort -rn
else
    echo "Aucune."
fi

echo
