# Tests d'intégration Cleaner et Nmap

Ces suites assemblent le vrai point d'entrée exporté (`get_library`), le routeur
SDK et la logique du plugin. Elles sont placées dans `tests/integration_tests.rs`
de chaque crate, séparément des tests unitaires dans `src/`.

## Exécuter

Depuis la racine du dépôt, sous Linux, avec le profil debug par défaut :

```sh
# Les 16 scénarios d'intégration (8 par plugin)
cargo test --locked -p griffon_cleaner -p griffon_nmap --test integration_tests

# Un seul plugin
cargo test --locked -p griffon_cleaner --test integration_tests
cargo test --locked -p griffon_nmap --test integration_tests

# Tests unitaires et d'intégration des deux plugins
cargo test --locked -p griffon_cleaner -p griffon_nmap --all-targets
```

Pas besoin d'installer Nmap, Docker ou curl. Les tests s'exécutent sans réseau,
sans accès au daemon Docker et sans privilèges supplémentaires.

Le job CI `test` les découvre automatiquement via `cargo nextest --workspace`.
Le script `scripts/plugin-coverage.sh` utilise `--all-targets` : sa mesure inclut
maintenant les exécutions unitaires **et** d'intégration. Les fichiers de tests
restent exclus du rapport. Le job `plugin-contract` reste indépendant.

## Isolation

Chaque scénario crée un `TempDir` puis lance un processus enfant du binaire de
test. Seul cet enfant reçoit un `HOME`, un `PATH` et un répertoire courant
pointant vers les fixtures. Le processus parent ne modifie pas son environnement,
ce qui permet l'exécution parallèle. Les fichiers temporaires sont nettoyés à la
fin et chaque scénario est limité à 20 secondes ; les journaux sont affichés en
cas d'échec.

Le test technique `isolated_session` apparaît comme « ignored » dans chaque
suite : c'est le point d'entrée enfant, explicitement lancé par chacun des huit
scénarios. Ce ne sont pas des scénarios métier laissés de côté. Ne pas lancer ce
helper directement avec `--ignored` : il exige l'environnement préparé par son
parent.

La suite exige un build debug car les chemins de configuration Cleaner et
d'historique Nmap y sont relatifs. Elle refuse un build release avant tout appel
au plugin, pour ne pas utiliser les chemins système de production.

## Scénarios Cleaner

- **Simulation** : requête `run_raw`, configuration temporaire, filtre `.log`,
  parcours réel du cache de fixture, comparaison des compteurs du rapport, de
  l'analyse et du JSON exporté ; aucun fichier supprimé.
- **Nettoyage réel de fixtures** : mêmes étapes, suppression des deux fichiers
  correspondants, conservation du JSON et d'un fichier hors cache ; un second
  passage ne trouve plus d'octets à nettoyer.
- **Sélection** : `list_candidates` → simulation de `delete_selected` → suppression
  effective ; tailles triées et compteurs cohérents, erreurs partielles pour un
  fichier hors périmètre et un lien symbolique, conservation de ces derniers.
- **Présentation** : cohérence entre `run_front`, `run_metrics` et les fichiers
  présents, métadonnées et mode simulation.
- **Erreurs** : paramètres et configuration invalides renvoyés en JSON, sans
  modification de fichiers ni lancement de commande Docker.

- **Docker en simulation safe/full** : configuration → commandes de disponibilité
  et d'estimation → rapports → actions frontend ; vérifie l'activation des volumes
  uniquement en full et l'absence d'appels de purge.
- **Échec partiel Docker** : une purge d'image simulée échoue, les suivantes se
  poursuivent ; ordre exact des commandes, exclusion des volumes en safe,
  compteurs de succès et erreur de permission dans les métriques.
- **Daemon Docker indisponible** : le rapport de cache reste exploitable, l'erreur
  de permission est propagée à l'analyse et aux métriques sans double comptage.

Seule la catégorie de cache utilisateur est activée, sous le HOME temporaire.
Un faux `docker` intercepte tous les appels. Les scénarios de simulation vérifient
qu'aucune purge n'est demandée ; les scénarios d'exécution contrôlent les arguments
de purge mais le script ne supprime aucune ressource Docker. `root_paths` ne sert pas à isoler le
nettoyage : le code de production utilise les chemins de cache connus.

## Scénarios Nmap

- **Scan et historique** : requête → vraie construction/lancement de commande →
  exécutable factice → parsing XML → réponse JSON → fichier d'historique. Vérifie
  les arguments exacts, les ports ouverts, les diagnostics, la suppression d'une
  entrée, le vidage et les relectures.
- **Exegol** : vérifie le préfixe `docker exec`, le conteneur choisi et la
  persistance de l'exécuteur ; Docker est remplacé par un script de fixture.
- **Échecs** : cible invalide rejetée avant lancement, code de sortie non nul,
  XML invalide, sortie non UTF-8, exécutable absent ; aucun historique créé.
- **Hôte inactif** : retour valide sans hôtes, conseil `-Pn` et diagnostics
  conservés à la relecture de l'historique.
- **Stockage** : limite de 100 résultats et ordre des entrées, historique corrompu
  préservé, erreur d'écriture reproductible sans dépendre des permissions root.

- **Sondes HTTP** : titre, en-têtes et robots.txt en local et Exegol, URL IPv6,
  arguments exacts, erreurs curl, exécutable absent et protocole rejeté avant
  lancement ; l'historique existant reste inchangé.
- **Sondes de services** : SSH, FTP, SMTP, DNS et SMB en local et Exegol,
  arguments et sorties vérifiés, récupération sur stderr, erreur de processus,
  action incompatible et exécutable absent ; historique inchangé.
- **Listing Exegol** : sélection par image ou label, rejet des lignes invalides
  et conteneurs sans rapport, liste vide, daemon indisponible et Docker absent.

Les scripts de fixture utilisent `/bin/sh` et `/bin/cat`, enregistrent les arguments
et renvoient des sorties déterministes. Ils ne lancent aucun scan.

## Limites

Ces suites testent les bibliothèques liées au binaire de test. Elles ne chargent
pas dynamiquement les `.so`, ne lancent pas le daemon ou l'interface graphique et
ne valident pas les comportements d'un vrai Nmap ou Docker. Ces vérifications
relèvent des suites de contrat, fonctionnelles et end-to-end distinctes.
