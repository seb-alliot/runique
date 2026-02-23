
## Résumé actuel des modification a apporter

## Configuration du mot de passe via le settings

## Status => a faire

    ## 1
       a) Ajouter un enum pour main.rs
        contenant False, auto => Argon2 | Bcrypt | autre , choix manuel

       b) Suivant l'option choisis de l'enum
         1) False => 0 hashage de mot depasse, le dev peux donc utiliser une api tiers
         2) si il choix manuel , la fonction de hash va récuperer l'option de la configuration et matcher dessus
         3) auto hashage ? => si option choisis ex auto::auto::argon2 , hash automatique du champs password ?
         4) verifi => fonction de verification du mot de pass qui va se basé sur l'option du hash pour savoir comment verifié
             ex => si Argon2 => verifi se base sur Argon2 , si bcrypt verifi se base sur bcrypt etc
             => si mode auto, verifi se base de la facon => choix de l'algorythme

## Status => a faire

    ## 2
       a) I18n
            => configuration sur main.rs via un choix de langue
            ex let lang = config.langage(enum possible)
                => charge un fichier json
       b) Tracing d'erreur
            => configurer le tracing sur le mode debug
                ex si debug=false
                        => tracing = off
                    si debug=true
                        => tracing = true
                        => rendu console et page de debug

## Status => a faire

    ## 3
        a) Finir le systeme de migration
            => une fois fini , api stable pour schema/migration
                => equivalent django => models/admin

        b) Vue admin
            b.1) ne plus basé le rendu admin sur les formulaires brute mais sur les models => ils font leur propre rendu
                    => les formulaires se base sur le model et non l'inverse si macro attribu connecter

            b.2) Permettre l'ajout de formulaire pour en recuperer la logique metier de l'api sur les models

## Status => a faire

    ## 4
        a) Middleware de csp
            => paufiner la configuration pour la rendre plus simple et lisible

        b) Stabilité
            b.1) checker toutes les features, tester exhaustif sur toutes les features

            b.2) Mettre a mal le framework, le poussé a bout pour la fiabilité

            b.3) trouver des failles pour les corrigers

            b.4) ne pas considerer le framework comme terminé, y a toujours a ajouter

## Status => fait

    ## 5
        a) paufiner/corriger les moteur de formulaire
            => double appele is_valide()
                premeire fois dans build_with_data

                deuxieme fois dans is_valide du handler du dev