#!/usr/bin/env bash
#
# Le job `demo` de la CI, en local, les moteurs en parallèle.
#
# Chaque moteur travaille dans sa propre copie du dépôt, avec son propre
# dossier de compilation : aucun ne voit ce que l'autre modifie. Dans une
# copie, les types de clé passent l'un après l'autre (scripts/demo_matrix.sh) :
# le Cargo.toml de demo-app change, cargo ne recompile que ce qui en dépend.
#
# Le dépôt lui-même n'est pas touché : ni demo-app, ni ses fichiers
# d'environnement, qui ne sont pas copiés (ils portent des secrets).
#
# Les copies sont dans target/ci-demo/copies/<moteur>/ et restent d'un passage
# à l'autre : seul le premier compile les dépendances. Chaque passage les
# remet à l'image du dépôt, modifications non commitées comprises.
#
# Les bases Postgres et MySQL de docker-compose.yml doivent tourner
# (`docker compose up -d --wait`).
#
# Usage :
#   bash scripts/demo_parallel.sh                  # 3 moteurs en parallèle, 3 clés chacun
#   bash scripts/demo_parallel.sh postgres mysql   # deux moteurs
#   bash scripts/demo_parallel.sh pk-uuid          # une clé, sur les 3 moteurs
#
# Suivre un moteur pendant le passage : tail -f target/ci-demo/logs/<moteur>.log

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

ENGINES=()
KEYS=()
for arg in "$@"; do
    case "$arg" in
        postgres | mysql | sqlite) ENGINES+=("$arg") ;;
        int | big-pk | pk-uuid) KEYS+=("$arg") ;;
        *)
            echo "argument inconnu : $arg (moteurs : postgres mysql sqlite ; clés : int big-pk pk-uuid)"
            exit 2
            ;;
    esac
done
[ "${#ENGINES[@]}" -eq 0 ] && ENGINES=(postgres mysql sqlite)
[ "${#KEYS[@]}" -eq 0 ] && KEYS=(int big-pk pk-uuid)

COPIES="$ROOT/target/ci-demo/copies"
LOGS="$ROOT/target/ci-demo/logs"
mkdir -p "$COPIES" "$LOGS" || exit 1

PIDS=()
# Chaque moteur est lancé dans son propre groupe de processus : l'arrêt
# atteint aussi les cargo qu'il a lancés.
stop_all() {
    local pid
    for pid in "${PIDS[@]}"; do
        kill -TERM -- "-$pid" 2>/dev/null
    done
}
trap 'stop_all; exit 130' INT TERM

for engine in "${ENGINES[@]}"; do
    copy="$COPIES/$engine"
    mkdir -p "$copy"
    # `target/` de la copie est gardé : c'est son cache de compilation.
    if ! rsync -a --delete \
        --exclude '/target/' --exclude '/.git/' --exclude '.env*' \
        "$ROOT/" "$copy/"; then
        echo "ECHEC: copie du dépôt pour $engine"
        stop_all
        exit 1
    fi
    log="$LOGS/$engine.log"
    DEMO_COMPOSE_ROOT="$ROOT" setsid bash "$copy/scripts/demo_matrix.sh" "$engine" "${KEYS[@]}" \
        > "$log" 2>&1 &
    PIDS+=("$!")
    echo "--- $engine lancé (journal : $log) ---"
done

status=0
for index in "${!ENGINES[@]}"; do
    wait "${PIDS[$index]}" || status=1
done

# Les journaux l'un après l'autre, pour que le bilan de chaque moteur se lise.
for engine in "${ENGINES[@]}"; do
    echo ""
    echo "================== journal $engine =================="
    cat "$LOGS/$engine.log"
done
exit "$status"
