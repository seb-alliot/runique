#!/usr/bin/env bash
#
# Le job `demo` de la CI, en local : demo-app compilée, migrée et testée pour
# chaque moteur et chaque type de clé, comme un vrai projet.
#
# Pour chaque couple :
#   1. une base neuve `ci_demo_<moteur>_<clé>` (jamais celle du site) ;
#   2. le `default = [...]` de demo-app/Cargo.toml réécrit avec le moteur et la clé ;
#   3. le fichier d'environnement du moteur (.env, .env.mysql ou .env.sqlite) ;
#   4. pour une clé autre que `int`, et pour SQLite : les migrations réécrites
#      depuis les modèles par `runique makemigrations` ;
#   5. `runique migration up`, puis `cargo test -p demo-app`.
#
# Tout ce que le script modifie est remis en place à la fin, même après un
# échec ou Ctrl-C : les Cargo.toml de demo-app et de son crate migration, ses
# trois fichiers d'environnement et son dossier migration/src. Rien n'est
# supprimé : ce qui est écarté va dans target/ci-demo/<date>/.
#
# Différence avec la CI : la commande n'est pas installée (cela remplacerait
# celle de la machine), elle est compilée une fois et appelée depuis target/.
#
# Les bases Postgres et MySQL de docker-compose.yml doivent tourner
# (`docker compose up -d --wait`). Les bases `ci_demo_*` restent après le
# passage ; le suivant les recrée.
#
# Les moteurs en parallèle, chacun dans sa copie du dépôt :
# scripts/demo_parallel.sh, qui appelle ce script une fois par moteur.
#
# Usage :
#   bash scripts/demo_matrix.sh                  # les 9 couples
#   bash scripts/demo_matrix.sh postgres         # un moteur, les 3 clés
#   bash scripts/demo_matrix.sh pk-uuid          # une clé, les 3 moteurs
#   bash scripts/demo_matrix.sh sqlite pk-uuid   # un seul couple

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

# Les conteneurs sont ceux du dépôt d'origine : lancé depuis une copie, le
# script doit les nommer par ce dossier, dont docker compose tire leur nom.
COMPOSE_ROOT="${DEMO_COMPOSE_ROOT:-$ROOT}"
compose() {
    docker compose --project-directory "$COMPOSE_ROOT" -f "$COMPOSE_ROOT/docker-compose.yml" "$@"
}

APP="demo-app"
MIG_SRC="$APP/migration/src"
MANIFESTS=("$APP/Cargo.toml" "$APP/migration/Cargo.toml")
ENV_FILES=("$APP/.env" "$APP/.env.mysql" "$APP/.env.sqlite")
RUNIQUE_BIN="$ROOT/target/debug/runique"

WORK="$ROOT/target/ci-demo/$(date +%Y%m%d-%H%M%S)"
SAVED="$WORK/saved"
mkdir -p "$SAVED/$APP/migration" || exit 1

# ── Sauvegarde ────────────────────────────────────────────────────────────────
# Les fichiers d'environnement sont déplacés, pas copiés : ils portent des
# secrets, et demo-app ne doit lire que celui du couple en cours.
for file in "${MANIFESTS[@]}"; do
    cp -p "$file" "$SAVED/$file" || exit 1
done
cp -rp "$MIG_SRC" "$SAVED/$APP/migration/src" || exit 1
for file in "${ENV_FILES[@]}"; do
    if [ -f "$file" ]; then
        mv "$file" "$SAVED/$file" || exit 1
    fi
done

# Écarte `$1` dans le dossier de travail, sous un nom qui ne se répète pas.
aside() {
    local target
    target="$WORK/aside-$(date +%s%N)-$(basename "$1")"
    mv "$1" "$target"
}

# demo-app telle qu'elle était avant le script, sauf ses fichiers
# d'environnement, qui restent à l'écart jusqu'à la fin.
pristine() {
    local file
    for file in "${MANIFESTS[@]}"; do
        cp -p "$SAVED/$file" "$file"
    done
    for file in "${ENV_FILES[@]}"; do
        [ -f "$file" ] && aside "$file"
    done
    aside "$MIG_SRC"
    cp -rp "$SAVED/$APP/migration/src" "$MIG_SRC"
}

restored=0
restore() {
    [ "$restored" -eq 1 ] && return
    restored=1
    pristine
    local file
    for file in "${ENV_FILES[@]}"; do
        [ -f "$SAVED/$file" ] && mv "$SAVED/$file" "$file"
    done
    echo "--- demo-app remise en état (fichiers écartés dans $WORK) ---"
}
trap restore EXIT

# ── La commande, compilée une fois ────────────────────────────────────────────
echo "--- compilation de la commande runique ---"
if ! cargo build -q -p runique-cli; then
    echo "ECHEC: compilation de runique-cli"
    exit 1
fi

# ── Un couple moteur × clé ────────────────────────────────────────────────────
run_pair() {
    local engine="$1" key="$2"
    local db="ci_demo_${engine}_${key//-/_}"
    local url env_file features
    local -a vars

    echo ""
    echo "================== demo-app ($engine, $key) =================="
    pristine

    case "$engine" in
        postgres)
            compose exec -T postgres dropdb --if-exists -U runique "$db" || return 1
            compose exec -T postgres createdb -U runique "$db" || return 1
            url="postgres://runique:runique_test@127.0.0.1:5433/$db"
            env_file=".env"
            vars=("DB_ENGINE=$engine" "DATABASE_URL=$url")
            ;;
        mysql)
            compose exec -T mysql mysql -uroot -proot -e \
                "DROP DATABASE IF EXISTS $db; CREATE DATABASE $db;" || return 1
            url="mysql://runique:runique_test@127.0.0.1:3307/$db"
            env_file=".env.mysql"
            vars=("DB_ENGINE=$engine" "DATABASE_URL=$url")
            ;;
        sqlite)
            # Un fichier neuf par passage : le dossier de travail est daté.
            local file="$WORK/$db.db"
            url="sqlite://$file?mode=rwc"
            env_file=".env.sqlite"
            # `DB_NAME` aussi : si `DATABASE_URL` ne répond pas, la connexion
            # se replie sur les clés `DB_*`, qui sans nom désignent
            # `local_base.sqlite`, une autre base.
            vars=("DB_ENGINE=$engine" "DATABASE_URL=$url" "DB_NAME=$file")
            ;;
    esac

    # Le moteur et la clé, là où un projet les écrit.
    features="\"$engine\""
    if [ "$key" != "int" ]; then
        features="$features, \"$key\""
        sed -i "s|^runique = { path = \"../../runique\" }|runique = { path = \"../../runique\", features = [\"$key\"] }|" \
            "$APP/migration/Cargo.toml"
        grep -q "\"$key\"" "$APP/migration/Cargo.toml" || {
            echo "ECHEC: la clé $key n'a pas pu être écrite dans $APP/migration/Cargo.toml"
            return 1
        }
    fi
    sed -i "s|^default = .*|default = [$features]|" "$APP/Cargo.toml"

    {
        echo "RUNIQUE_ENV=dev"
        echo "DEBUG=true"
        printf '%s\n' "${vars[@]}"
        echo "SECRET_KEY=ci-only-key-with-at-least-32-characters"
    } > "$APP/$env_file"

    # Les migrations du dépôt sont écrites pour des clés `int`, et l'une d'elles
    # modifie une clé étrangère sur une table existante, ce que SQLite ne fait
    # pas : dans ces cas elles sont réécrites depuis les modèles.
    if [ "$key" != "int" ] || [ "$engine" = "sqlite" ]; then
        local old="$WORK/migrations-$db"
        mkdir -p "$old"
        find "$MIG_SRC" -maxdepth 1 \
            \( -name 'm[0-9]*.rs' -o -name snapshots -o -name lib.rs \) \
            -exec mv -t "$old" {} +
        echo "--- makemigrations ---"
        (cd "$APP" && env "${vars[@]}" "$RUNIQUE_BIN" makemigrations) || return 1
    fi

    echo "--- migration up ---"
    (cd "$APP" && env "${vars[@]}" "$RUNIQUE_BIN" migration up) || return 1

    echo "--- cargo test -p demo-app ---"
    env "${vars[@]}" cargo test -p demo-app || return 1
}

FAILED=()
for engine in "${ENGINES[@]}"; do
    for key in "${KEYS[@]}"; do
        if run_pair "$engine" "$key"; then
            echo "OK: demo-app ($engine, $key)"
        else
            echo "ECHEC: demo-app ($engine, $key)"
            FAILED+=("$engine, $key")
        fi
    done
done

echo ""
echo "================== demo-app : bilan =================="
if [ "${#FAILED[@]}" -eq 0 ]; then
    echo "OK: ${#ENGINES[@]} moteur(s) × ${#KEYS[@]} clé(s)"
    exit 0
fi
printf 'ECHEC: demo-app (%s)\n' "${FAILED[@]}"
exit 1
