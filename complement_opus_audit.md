# Audit de la documentation Runique

**Périmètre :** `https://runique.io/docs/en` et `https://runique.io/docs/fr`
**Date :** 3 juin 2026
**Méthode :** crawl manuel des deux index, des 17 sections de premier niveau EN, de toutes les feuilles `installation/`, de plusieurs feuilles profondes (`model`, `auth`, `admin`), et contrôles de parité FR. Tous les liens signalés comme à risque ont été testés (résolution 200 / 404).

---

## Résumé exécutif

Sur le fond, la documentation est solide : prose claire, exemples de code réels et cohérents, mises en garde opérationnelles pertinentes (ACME / port 80 / `CAP_NET_BIND_SERVICE`, nuance HTTP/2 `:authority`, limitation « field override not yet supported » avec workaround, section « what's still missing » du comparatif). 

**Tous les défauts trouvés sont mécaniques / générés, pas rédactionnels.** Ils se regroupent en quatre familles :

1. **Liens cassés (404)** — 4 confirmés, tous côté EN, tous dans des tableaux de contenu.
2. **Liens mal aiguillés (200 mais mauvaise cible)** — ancres/pages distinctes aplaties sur une URL de base.
3. **Bandeau de version incohérent** — 5 valeurs distinctes selon la page.
4. **i18n + génération de sommaire incomplets** — libellés non traduits, sommaire qui saute la 1ʳᵉ section.

---

## 1. Liens cassés confirmés (404)

Tous sur le **côté EN**, tous dans des **tableaux de corps de page** (la barre latérale, elle, est correcte).

| Lien mort (404) | Page qui le contient | Bonne cible |
|---|---|---|
| `/docs/en/model/formulaires` | tableau de `/docs/en/model` | `/docs/en/model/forms` |
| `/docs/en/model/forms/forms` | renvoi depuis `/docs/en/orm` | `/docs/en/model/forms` |
| `/docs/en/auth/modele` | tableau de `/docs/en/auth` | `/docs/en/auth/model` |
| `/docs/en/auth/exemple` | tableau de `/docs/en/auth` | `/docs/en/auth/example` |

### Cause racine

Côté **FR**, barre latérale et tableau sont cohérents : `/docs/fr/auth/modele` et `/docs/fr/auth/exemple` existent et résolvent (200). Côté **EN**, la barre latérale utilise les bons slugs anglais (`model`, `example`, `forms`) mais **les tableaux ont conservé les slugs français** (`modele`, `exemple`, `formulaires`) → 404.

> **Diagnostic :** les pages EN ont été générées/traduites depuis le FR. La traduction des *libellés* est faite, mais celle des *slugs de liens internes présents dans le contenu Markdown* ne l'est pas. À corriger dans le pipeline de génération (re-slugification des liens internes lors de la traduction).

---

## 2. Liens mal aiguillés (200 mais mauvaise cible)

Pas des 404, mais ils mènent à la mauvaise page — plusieurs ancres ou pages distinctes sont écrasées sur une même URL de base.

- **`/docs/en/architecture`** — barre latérale : « Key Concepts » **et** « Makemigrations Internals » pointent tous deux vers `/architecture/concepts`. Le second devrait viser une page dédiée (ex. `/architecture/makemigrations`).
- **`/docs/en/middleware`** — barre latérale : **5 entrées** (CSP, CSP Directives, Security Headers, CSP Nonce, CSP Profiles) pointent **toutes** vers `/middleware/csp`. Ce sont visiblement des ancres (`/middleware/csp#nonce`, etc.) aplaties sur la page de base.
- **`/docs/en/admin`** — tableau : « CLI », « Daemon & generation » et « Macro `admin!` » pointent tous vers `/admin/declaration`. Or `/admin/declaration-daemon` et `/admin/declaration-macro` existent (vérifié, 200). Le bug se répète sur `/admin/declaration-daemon`, dont la table « Related sections » renvoie « Macro `admin!` » → `/admin/declaration`.

---

## 3. Bandeau de version incohérent

Le numéro affiché dans le header varie selon la page :

| Valeur affichée | Où |
|---|---|
| `v2.1.14 post release` | index EN/FR, pages régénérées (`model/forms`, `admin/declaration-daemon`, `env`) |
| `v2.1.7` | `comparatif` |
| `v2.1.6` | majorité des sections EN (`installation/*`, `architecture`, `configuration`, `routing`, `orm`, `model`, `formulaire`, `template`, `flash`, `middleware`, `auth`, `session`, `admin`, `exemple`, `mailer`) |
| `v2.1.5 - Upcoming` | pages FR (`installation/prerequis`, `auth`, `auth/modele`) |
| `2.1.5` | en dur dans les `Cargo.toml` d'exemple (`database`, `troubleshooting`) |

Le bandeau n'est pas centralisé : seules certaines pages ont été régénérées à 2.1.14, le reste est figé sur une valeur antérieure.

---

## 4. Sommaire « On this page » défaillant

### Saute le premier `##`

Le mini-sommaire omet la première section `##` de la page. Confirmé sur :

- les 5 feuilles `installation/*` (`prerequisites`, `database`, `migrations`, `network`, `troubleshooting`)
- `comparatif` (saute « CLI »)
- `auth/model` et `auth/modele` (saute « Built-in Model » / « Modèle built-in »)

**Corrélation nette :** les pages **non régénérées (v2.1.6 / v2.1.7)** ont le bug ; les pages **v2.1.14** ne l'ont pas. C'est probablement un bug de générateur déjà corrigé, mais ces pages n'ont pas été reconstruites.

### Autre variante

- `/docs/en/formulaire` : « Overview » apparaît **deux fois** dans le sommaire.

---

## 5. i18n incomplet

- Le libellé **« On this page »** reste en anglais sur **toutes** les pages FR (devrait être « Sur cette page »).
- Les **en-têtes de groupe** de l'index FR restent en anglais : « Getting Started », « Routing & Web », « Database », « Security & Auth ». Seul « Autres » est traduit.
- **« Comparatif »** n'est pas traduit sur l'index EN (devrait être « Comparison »).

---

## 6. Autres finitions

- **Tableau ↔ barre latérale** : `/docs/en/configuration` liste « Structured Tracing » dans la barre latérale mais l'oublie dans le tableau récapitulatif.
- **Doublon de contenu probable** : `/docs/en/env/*` et `/docs/en/configuration/variables` couvrent tous deux les variables d'environnement.
- **Fuite de fichier source** : `/docs/en/model` cite « This document (`12-model.md`) » dans l'ordre de lecture conseillé.
- **Drift FR/EN mineur** : le `.env` de `fr/installation/prerequis` contient une ligne de commentaire (`# Les hosts autorisés se configurent dans le builder…`) absente de la version EN. Les deux langues semblent maintenues indépendamment.
- **À vérifier** : le banner de démarrage est montré traduit en FR (« 🦀 Runique Framework opérationnel »). Le binaire localise-t-il réellement sa sortie, ou la doc montre-t-elle un output que l'utilisateur ne verra jamais en français ?

---

## 7. Sitemap EN (barre latérale = source fiable)

```
installation/  index prerequisites database cli migrations network troubleshooting
architecture/  index concepts lifecycle macros middleware tera testing      [+ page "makemigrations" manquante ?]
configuration/ index builder code i18n password tracing variables
env/           index application assets security
routing/       index extractors macros responses
formulaire/    index errors example fields helpers prisme templates trait
template/      index filters forms syntax tags
flash/         index handlers macros templates
orm/           index advanced manager queries
model/         index dsl forms generation
middleware/    index builder cors csp csrf hosts-cache login-required
               open-redirect permissions-policy rate-limit sessions trusted-proxies
auth/          index example login-guard middleware model password-reset session
session/       index protection store usage
admin/         index declaration declaration-daemon declaration-macro evolution list
               permission setup template-clef template-csrf template-surcharge template user-creation
exemple/       index forms minimal others upload
mailer/        index
comparatif/    index
```

Le FR reproduit cette structure avec slugs localisés (`installation/base-de-donnees`, `installation/prerequis`, `auth/modele`, `auth/exemple`, `model/formulaires`, etc.) et s'est révélé cohérent partout où il a été testé.

---

## 8. Recommandations priorisées

| Priorité | Action |
|---|---|
| **Haute** | Corriger les 4 liens 404 EN (re-slugifier `modele`→`model`, `exemple`→`example`, `formulaires`→`forms` dans les tableaux et le renvoi ORM). |
| **Haute** | Régénérer toutes les pages pour uniformiser le bandeau de version sur `v2.1.14` (et corriger en cascade le sommaire « skip 1ᵉʳ `##` »). |
| **Moyenne** | Réparer les liens mal aiguillés : ancres CSP du `middleware`, `declaration-daemon`/`declaration-macro` dans `admin`, dédoublonnage `architecture/concepts`. |
| **Moyenne** | Traduire « On this page » → « Sur cette page » et les en-têtes de groupe de l'index FR ; traduire « Comparatif » sur l'index EN. |
| **Basse** | Fusionner ou différencier `env/*` et `configuration/variables` ; retirer la fuite `12-model.md` ; corriger le double « Overview » de `formulaire`. |

---

## Annexe — URLs testées explicitement

**Résolvent (200) :** index EN/FR ; `installation` (index + prerequisites, database, cli, migrations, network, troubleshooting) ; `architecture`, `configuration`, `env`, `routing`, `formulaire`, `template`, `flash`, `orm`, `model`, `middleware`, `auth`, `session`, `admin`, `exemple`, `mailer`, `comparatif` (index de chaque) ; `model/forms`, `auth/model`, `admin/declaration-daemon` ; FR : `installation` (index), `installation/prerequis`, `auth` (index), `auth/modele`.

**404 :** `model/formulaires`, `model/forms/forms`, `auth/modele` (EN), `auth/exemple` (EN).

> Le balayage 200/404 littéral de l'intégralité des ~160 URL (toutes feuilles × 2 langues) n'a pas été effectué : les feuilles « barre latérale » échantillonnées renvoient toutes 200, et la cause racine des 404 est identifiée. Un balayage exhaustif d'une langue complète peut être fait sur demande.