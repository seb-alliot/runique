### 1 pseudo code

macro admin! qui prendrais le chemin du formulaire en paramatre
remplirais un registre

exemple =>

pub struct User()
admin! (forms::user)

    champs ...

remplis un vecteur de formulaire register_for_admin() par exemple avec (forms::user)

### 2 routeur admin

## 1 page de connection

middleware d'auth connecté basé sur le superuser intialement, modifiable dans la vue admin pour surcharger via des role du dev

## 2 formulaire

boucle sur register_for_admin() pour remplir sur le template admin via un for form in register_for_admin pour afficher tous les formulaires, la mise en page s'occupperai de separer les form via des selecteur par key, value

## 3 connection a la bdd

la vue admin devras etre connecter via le role superuser initialement donc avec les droits qui vont avec, la creation de role devras modifier se style de droit si moderateur est creer , ou un admin simple ?



## Gestion accés au formulaire via le role

ajouter un filtre via un role => django le gere via groupe il me semble => reproduire se principe puis dans le template ajouter un if user.is_admin && permission == "nom des permission"

## Recap actuel

1 => middleware auth, verifis si l'auth est connecter, si oui verifier les permission Admin => accé si non retour sur la page de login avec msg d'erreur vous n'avez pas les droits

2 => suivant le role des permissions, affichage des formulaires => de cette facon pas de droit a gerer sur la connection de la bdd, sa se fait via le connection => plus simple a maintenir

3 => ajouter la table user sur toutes les pages de gestion de formulaire => necessaire dans 90% des cas d'usage si ce n'est plus

4 => le js reliant le user au fomrulaire se feras pas le dev qui liera le js au formulaire via add_js() => query db sains normalement pour toutess requete fetch

5 => le formulaire d'accé au user devras avec une  requete de base pour afficher tous les users => defi => sea orm, le nom de la table dois etre imposer pour le bon fonctionnement => défi => un dev peux changer le nom de la table => solution actuel => imposer une convention de nommage pour la table user et les permissions

6 => j'ai oublié quoi?

 => requete fetch sur la table user, formulaire user personnalisé (select, recherche par nom) relier a la struct du dev
    => convention de nommage imposer table users
    => nommer les formulaires pour en ajouter un titre d'accé dans l'admin => exemple forms user titre "Compte user" html vue admin Compte user et sa afficher la vue de recherche d'un compte user

=> Gerer les erreurs renvoyé par la bdd => exemple => user supprimé => ne pas crash si d'autres tables dépendent de cet element, a uniformisé pour toutes les tables

7   => fonction de comparaison dans le register_admin_form => si la macro admin! est utilisé , la fonction analyse les champs de la struct models et le formulaire qui lui est lié, si difference de champs, alerte de couleur sinon rien, la base etant le models => source de vérité absolu
    => cargo watch ? avec un message d'erreur sur le champs en trop ou manquant ? => macro donc reactivité en temps reel
    => surveiller les doublons de formulaire declarer sur un model => interdiction de doublon ?
    => la fonction de comparaison ne s'active qu'a l'activation de la vue admin dans le router via with_admin(true)
    => macro qui lis un admin.rs qui contiendrais la liste de register_admin_form pour facilité le parse ?