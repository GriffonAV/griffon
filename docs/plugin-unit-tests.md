# Tests unitaires Cleaner et Nmap

## Exécuter

Depuis la racine du dépôt, sous Linux :

```sh
cargo test --locked -p griffon_cleaner -p griffon_nmap --all-targets
cargo clippy --locked -p griffon_cleaner -p griffon_nmap --all-targets -- -D warnings
cargo fmt -p griffon_cleaner -p griffon_nmap --check
```

Les tests ne nécessitent ni Nmap, ni curl, ni Docker, ni droits root, ni réseau.
Les fichiers créés/supprimés sont dans des `tempfile::TempDir`, nettoyés en fin de test.
Aucun changement global de `HOME`, `PATH` ou du répertoire courant n'est effectué :
les tests restent exécutables en parallèle. Les tests de processus et de liens
symboliques ciblent Linux, comme les plugins.

## Couverture reproductible

```sh
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov --version 0.9.1 --locked
bash scripts/plugin-coverage.sh
```

Sorties : `target/plugin-coverage/coverage.json` et
`target/plugin-coverage/html/index.html`.

Pour ouvrir le rapport HTML dans le navigateur par défaut, après la génération,
exécuter depuis la racine du dépôt :

```sh
xdg-open target/plugin-coverage/html/index.html
```

Le rapport présente la couverture par fichier. Cliquer sur un fichier pour
consulter son code et repérer les lignes exécutées ou non par les tests : cela
permet d'identifier les cas à ajouter. Le JSON sert au contrôle automatique des
seuils ; le HTML facilite l'exploration des résultats.

Sans `xdg-open`, ouvrir directement le fichier `index.html` dans un navigateur.
Pour consulter un rapport téléchargé depuis la CI, extraire l'archive complète
et conserver les fichiers et sous-dossiers HTML ensemble afin que les liens
vers les pages de code fonctionnent.

Le seuil est **70 % des lignes exécutables, séparément pour chaque plugin**.
Le calcul additionne les lignes couvertes et mesurées des fichiers de production
sous `plugins/<plugin>/src/`; il ne moyenne pas les pourcentages des fichiers.
Il inclut le `main.rs` de Cleaner (actuellement non couvert) et les modules
inactifs `logs`, `packages` et `bigfiles`. Seuls les fichiers réservés aux tests
(`tests.rs`, `*_tests.rs`) sont exclus. Les dépendances partagées ne sont pas
comptées dans le score des plugins. Aucune exclusion de code métier.

La CI contient un job autonome `plugin-coverage`, sans dépendances GTK/WebKit.
Il exécute les tests, contrôle chaque seuil et conserve le JSON et le HTML.
Les douze tests Nmap préexistants, dont les quatre tests de diagnostics locaux,
sont conservés dans `tests.rs` et `feedback_tests.rs`.

## Périmètre testé

| Plugin | Cas principaux |
| --- | --- |
| Cleaner | Configuration valide/invalide, alias et formats des filtres, priorité du mode simulation, normalisation des extensions, priorité `paths`/`items` |
| Cleaner | Agrégation des modules et propagation des erreurs, calcul des taux et classements, décompte des permissions sans doublon, exports JSON et présentation frontend |
| Cleaner | Candidats triés, tailles récursives, suppressions réelles/simulées sur fixtures, filtrage, échecs partiels, refus des racines protégées, chemins hors périmètre, liens symboliques et traversées |
| Cleaner | Plans Docker safe/full, aucune purge en simulation, volumes exclus du profil safe, CLI/daemon indisponibles, succès et erreurs simulés |
| Nmap | Cibles/ports/conteneurs, limites et entrées hostiles, profils/options, arguments distincts pour exécution locale et Exegol |
| Nmap | XML complet/incomplet/invalide, hôtes inactifs, diagnostics, sortie non UTF-8, processus en erreur ou absent, échec de persistance |
| Nmap | Historique temporaire, limite de 100, suppression, vidage, format ancien/corrompu, listing Exegol, sondes HTTP et services simulées |

Les fonctions publiques et l'ABI restent identiques. Les petits helpers internes
acceptent un exécuteur de commandes, un chemin d'historique ou des racines de
cache explicites. La production continue d'utiliser `Command::output`, son
historique et les chemins de cache connus.

## Suite : intégration, fonctionnel, end-to-end

1. **Intégration des plugins et du daemon** : prolonger les tests existants de
   `daemon/plugin_host/tests` pour charger les `.so`, vérifier les manifestes,
   le routage des messages et les réponses sérialisées. Ajouter des fixtures
   d'historique et tester les erreurs de stockage. Couvrir ensuite les écritures
   concurrentes et l'unicité des identifiants d'historique.
2. **Fonctionnel Cleaner** : lancer le binaire/daemon dans un conteneur ou une VM
   jetable avec une arborescence connue ; comparer exactement les fichiers avant
   et après, puis les compteurs et le rapport. Vérifier dry-run, filtres,
   permissions et safe/full. Attention : `ExecutionContext.root_paths` n'isole
   pas actuellement les opérations sur les caches connus. Ne pas utiliser cette
   option seule comme protection d'un test fonctionnel sur la machine hôte.
3. **Fonctionnel Nmap** : service TCP/HTTP déterministe sur loopback dans un réseau
   de test isolé, binaire Nmap réel, vérification des ports/services et de
   l'historique. Exegol/Docker dans une suite optionnelle distincte. Aucun scan
   d'hôte public ; ports dynamiques et délais bornés.
4. **End-to-end GUI → IPC → daemon → plugin** : scénario de simulation Cleaner,
   sélection/suppression d'une fixture, scan local Nmap, consultation puis
   suppression de l'historique ; vérifier les diagnostics visibles dans l'UI.

Les adaptateurs système réels, le point d'entrée CLI Cleaner et certaines erreurs
rares de filesystem restent à couvrir dans ces suites. La couverture de lignes
ne mesure pas toutes les branches, ni les situations de concurrence.
