# Flux — Cycle requête, CSRF et upload

## Séquence : POST formulaire (multipart) → handler

```mermaid
sequenceDiagram
    participant C as Client
    participant MW as csrf_middleware
    participant EX as Request::from_request
    participant P as prisme_pipeline
    participant AG as aegis / parse_multipart
    participant H as Handler
    participant F as request.form(nom)
    participant V as ValidationForm

    C->>MW: POST multipart (csrf_token en champ, pas de header)
    alt Sec-Fetch-Site = cross-site ou same-site (hors csrf_exempt)
        MW-->>C: 403 Cross-site form submission
    else même origine, ou en-tête absent
        MW->>EX: next.run()
        EX->>P: prisme_pipeline()
        P->>AG: aegis(req) selon le content-type
        AG->>AG: fichier écrit en .staging-uuid, jamais servi (jeton valide requis)
        AG-->>P: StrVecMap (chemins en staging)
        P->>P: check_csrf() pose csrf_valid (ne rejette pas)
        P-->>EX: Prisme { data, csrf_valid, staging }
        EX-->>H: Request (prisme posé)
        H->>F: request.form(nom)
        Note over F: nom, paramètres d'URL, nonce CSP, Gated si route admin, honeypot
        alt la requête ne renvoie pas ce nom (sent_by faux)
            F-->>H: form vierge, sans erreur
        else c'est son envoi
            Note over F: honeypot ou CSRF KO : force_invalid + message<br/>puis fill(data)
            F-->>H: form rempli
        end
        H->>V: try_form(form, &request)
        Note over V: keep_submitted_if(sent)<br/>allow_get/allow_post puis is_valid()
        V-->>H: Ok(validé) ou Err(form)
    end
```

## Séquence : `save` d'un formulaire avec fichier

```mermaid
sequenceDiagram
    participant H as Handler
    participant V as ValidationForm::save
    participant T as Transaction
    participant FS as Fichiers (staging puis MEDIA_ROOT)

    H->>V: save(&db), db = connexion ou transaction du handler
    Note over V: changes() : par colonne, champ non verrouillé → to_value,<br/>sinon code_values (add_value hors champ), sinon NotSet
    alt route déclarée admin
        Note over V: gated_write : as_adb() ou refus InTransaction, puis save_as (porte)
    else hors admin
        V->>T: begin + écriture de la ligne
        alt écriture refusée
            T-->>V: rollback
            Note over FS: le fichier reste en staging, retiré à la fin de la requête
        else écriture acceptée
            V->>FS: publish_uploads (staging vers destination)
            V->>T: commit
            alt commit OK
                V->>FS: complete_uploads (l'ancien fichier reste)
            else commit KO
                V->>FS: rollback_uploads (le fichier repart)
            end
        end
    end
    V-->>H: Ok(form) ou Err(form) avec l'erreur posée
```

## Points clés du flux

- **`csrf_middleware`** ([csrf.rs:75](../../runique/src/middleware/security/csrf.rs#L75)) :
  - GET/HEAD : strip `csrf_token` de l'URL (anti-fuite en query) + 302.
  - POST/PUT/DELETE/PATCH avec header `X-CSRF-Token` → valide en `ct_eq`, 403 si KO.
  - POST **form HTML sans header** : refusé (403) si le navigateur annonce un autre site
    (`from_another_site`, [csrf.rs:203](../../runique/src/middleware/security/csrf.rs#L203)),
    sinon délégué à Prisme/`form()`.
  - JSON sans header → 403 `CSRF token required`.
  - `csrf_exempt` saute ces contrôles ; le jeton reste généré.
- **`prisme_pipeline`** ([extractor.rs:63](../../runique/src/forms/extractor.rs#L63)) : aegis
  (parse) → `check_csrf` (flag seulement, **ne rejette pas**). Sentinel et `GuardRules` n'existent plus.
- **Enforcement réel** : `request.form(nom)` ([template.rs:467](../../runique/src/context/template.rs#L467))
  pose `force_invalid = true` si CSRF KO → `is_save_allowed()` renvoie false, et
  pousse un message (`t("csrf.invalid_or_missing")`) dans `Forms.errors`. Seul le formulaire
  dont le nom revient (`runique_form`) subit ce contrôle : les autres de la page restent vierges.
- **Handler** : `ValidationForm::try_form(form, &request).await` dispatche sur
  `allow_get`/`allow_post` selon `request.method.is_safe()`. Détail :
  [../uml/forms/formulaires.md](../uml/forms/formulaires.md).
- **`csrf.js`** : `request.render()` ajoute une seule balise par page contenant un champ
  `csrf_token` (`with_csrf_script`, [template.rs:244](../../runique/src/context/template.rs#L244)),
  aucune si la page le charge déjà.

## Anomalies / flux suspects

### 🔴 C1 — Upload COMMIT en MEDIA_ROOT avant CSRF/validation — ✅ CORRIGÉ
**Corrigé (2.1.21).** `parse_multipart` écrit désormais en **staging** `.staging-{uuid}` (non
servi) ; `FileField::finalize` est le seul committer (après CSRF + validation). Description du
problème d'origine ci-dessous (conservée pour mémoire) :
[`parse_html.rs:166-184`](../../runique/src/utils/forms/parse_html.rs#L166)
`parse_multipart` ne fait pas que streamer en tmp : il **rename immédiatement tmp →
`MEDIA_ROOT/uuid.ext`** à la fin du parsing, et renvoie les chemins finaux. Or `parse_multipart`
tourne dans `prisme_pipeline`, exécuté par `Request::from_request` sur **toute** requête
multipart, **avant** :
- la validation CSRF (`csrf_valid` calculé après `aegis`),
- la validation du formulaire (`is_valid`),
- le check d'extension (qui vit dans `FileField::validate`, plus tard).

Conséquences :
1. **Écriture de fichier non authentifiée** sur tout endpoint public extrayant `Request` et
   acceptant du multipart : le fichier est commité en MEDIA_ROOT quoi qu'il arrive.
2. **Aucun filtre d'extension** à ce stade (seulement la taille) → un `.html`/`.svg`/`.js`
   peut atterrir dans MEDIA_ROOT ; si MEDIA_ROOT est servi en statique, risque de **stored
   XSS**/contenu hostile selon le content-type servi.
3. **Remplissage disque** non authentifié (borné `max_upload_mb` par fichier, mais répétable).
Seules les routes admin sont protégées car `admin_required` (middleware) s'exécute **avant**
l'extraction `Request`. Les routes publiques n'ont pas ce filet.

### 🟠 C2 — CSRF des forms HTML reposait entièrement sur `req.form()` (footgun) — ✅ CORRIGÉ
**Corrigé (2.1.21).** `Prisme::data` est passé en `pub(crate)` : le code tiers ne peut plus
lire le corps brut hors CSRF. Seuls `req.form()` et `req.prisme.checked_data()` (fail-closed,
renvoie `None` si CSRF KO — pour les endpoints API/AJAX) exposent le corps. La politique de
méthode CSRF est centralisée dans `csrf_required()` (source unique, partagée par le pipeline
et `form()`). Audit interne clean (admin/login/`form()` gardent tous la CSRF avant `.data`).

### 🔴 C3 — Aucun rollback du fichier commité quand la requête est rejetée — ✅ CORRIGÉ
**Corrigé (2.1.21).** Conséquence de C1, résolue par le même fix : le staging non commité est
purgé par `sweep_stale_staging` (TTL), échecs tracés (jamais avalés). Plus de fichier orphelin
en MEDIA_ROOT après un rejet (CSRF/honeypot/validation).

### 🟡 C4 — `csrf_gate` : module mort / commentaire trompeur — ✅ CORRIGÉ
**Corrigé (2.1.21).** Le module `csrf_gate` était vide (poids mort) ; la logique est dans
`extractor::check_csrf`. Module supprimé (fichier + `pub mod` + tombstone test) et commentaire
`form.rs:194` corrigé pour pointer le vrai chemin.

### 🟡 C5 — Méthodes sûres : seuls GET/HEAD exemptés — ✅ DURCI
**Durci (2.1.21).** Politique CSRF par méthode collapsée en source unique `csrf_required()`
(GET/HEAD exemptés, tout le reste exige un token, fail-closed), partagée par le pipeline et
`Request::form()` → plus de dérive possible entre les deux sites. Comportement identique.

### 🟠 C6 — Échec CSRF totalement silencieux (aucun message) — ✅ CORRIGÉ (2026-09-22)
`force_invalid` court-circuite `is_valid()` avant que `HiddenField::validate()`
n'atteigne sa propre branche de message d'erreur CSRF — branche **morte** de toute
façon, car elle dépendait de `set_expected_value()`, jamais appelé en production.
Un CSRF invalide ou expiré (TTL session anonyme, cf.
[../uml/middleware/sessions.md](../uml/middleware/sessions.md)) faisait donc
revenir le formulaire **vide, sans aucune explication** — trouvé en testant en
conditions réelles `ValidationForm<F>` (pas un problème introduit par lui).
Corrigé au niveau framework, dans `Request::form()` : `t("csrf.invalid_or_missing")`
est poussé dans `Forms.errors` dès la détection, avant le court-circuit — bénéficie
à toute app Runique, pas seulement demo-app. Doublé d'un rafraîchissement JS du
token au `submit` (`runique/static/js/csrf.js`) pour réduire l'occurrence réelle
(TTL session anonyme par défaut 10 min).

### 🟠 C7 — Formulaire d'un autre site vers un handler à extracteur Axum — ✅ DURCI (2026-11-04)
Un handler lisant son corps avec `Form<T>`/`Multipart` ne vérifie pas le jeton. Le middleware
refuse désormais (403) un formulaire annoncé `cross-site` ou `same-site` par `Sec-Fetch-Site`,
toujours actif. Reste ouvert : une requête sans cet en-tête (HTTP hors localhost, ancien
navigateur) n'est arrêtée que par un jeton que ce handler doit vérifier lui-même. Sorties :
`csrf_exempt`, ou `X-CSRF-Token` par `fetch`.

### 🟠 C8 — Fichier et ligne d'un même `save` désaccordés — ✅ CORRIGÉ (2026-11-02)
Le fichier est déplacé dans la transaction, entre l'écriture et le commit (`write_with_uploads`,
[form.rs:65](../../runique/src/forms/form.rs#L65)) : une écriture refusée ne publie rien, et un
lecteur ne voit jamais de ligne sans son fichier. Risque résiduel : arrêt brutal ou commit perdu
après le déplacement, ou transaction du handler quittée sans commit → fichier orphelin, jamais de
lien cassé. Runique ne supprime plus le fichier remplacé.

### 🟠 C9 — Page à plusieurs formulaires — ✅ CORRIGÉ (2026-11-03)
Remplir chaque formulaire depuis le même corps recopiait les valeurs et l'erreur CSRF d'un
formulaire sur l'autre. `request.form(nom)` ne remplit que si le nom revient (`sent_by`) ;
`keep_submitted_if` retire `is_submitted()` à un formulaire prérempli par le handler.
