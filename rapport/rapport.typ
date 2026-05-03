#import "@local/polytech:1.0.0": *;

#set text(size: 14pt)

#show raw.where(block: false): it => box(
  fill: luma(220),
  inset: (x: 4pt, y: 2pt),
  radius: 3pt,
  baseline: 2pt,
  it,
)

#show: conf(doctitle: "VRPTW", subject: "Optimisation discrète", theme: blue)[
  #titlepage(authors: "Clément RENIERS")

  = Préambule : Exécution du code

  == Installation de Rust

  Pour installer Rust, vous pouvez suivre les instructions sur le site officiel : https://www.rust-lang.org/tools/install. Sur un système Linux ou MacOS, il faut exécuter la commande suivante :

  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

  Cette commande téléchargera plusieurs programmes :

  - Rustup, l'outil de gestion de Rust
  - Cargo, le gestionnaire de paquets de Rust
  - Rustc, le compilateur de Rust

  == Exécution du code

  Pour exécuter le code, il suffit de se placer à la racine du projet (là où se trouve le fichier `Cargo.toml`), et d'exécuter la commande suivante :

  ```bash
  cargo run -r
  ```

  Cargo s'occupe de l'installation de toutes les dépendances, de la compilation du code, et de son exécution#footnote[L'option `-r` de la commande indique à Cargo de compiler le code en mode release, pour des performances maximales]#footnote[La première compilation peut prendre beaucoup de temps : Cargo doit télécharger tous les paquets nécessaires, dont les bibliothèques graphiques.].

  == Recommandation

  Pour bien visualiser le code dans les éditeurs de texte / IDE, je recommande d'utiliser `rust-analyzer`, un plugin pour la plupart des IDE qui peremt d'avoir une meilleure compréhension du code Rust.

  = Introduction

  Ce rapport vise à présenter la résolution du problème de tournées de véhicules avec fenêtres de temps#footnote("Cette traduction est celle de Wikipédia, bien que je ne la trouve pas très parlante. On parlera de VRPTW pour désigner le problème") (VRPTW).

  Dans ce document, je vais d'abord présenter rapidement le problème, puis ma vision de la résolution. Ensuite, j'expliquerai mon choix de technologies et de méthodes. Enfin, je montrerai les résultats obtenus.

  = Présentation du problème

  == VRP

  Le problème de tournées de véhicules (VRP) est un  problème d'optimisation qui consiste à trouver les itinéraires optimaux pour un lot de camions qui doivent livrer des marchandises à un ensemble de clients.
  Les camions ont une capacité limitée, donc un camion ne peut pas livrer à tous les clients. Le but est donc de minimiser la distance totale parcourue par les camions.

  == VRPTW

  Le VRPTW est une extension du VRP qui ajoute des contraintes de temps à chaque livraison. On imagine que chaque client n'est présent que dans une certaine fenêtre de temps, que les camions doivent respecter.

  = Choix de la technologie

  == Langage de programmation

  Pour résoudre ce problème, la solution naturelle est d'utiliser un langage de bas niveau, qui permet de manipuler du code au plus près du langage machine. J'ai pour ça choisi Rust : C'est un langage récent qui permet d'écrire du code rapide et efficace, avec une syntaxe plus proche de celle de Python que celle de C++. De plus, Rust a une excellente gestion de la mémoire, intéressant pour ce genre de projet.
  #figure(
    grid(
      columns: 2,
      gutter: 50pt,
      [
        ```cpp
        #include <iostream>
        #include <vector>

        int main() {
          std::vector<int> values{1, 2, 3, 4, 5, 6};
          int sum = 0;

          for (int v : values) {
            if (v % 2 == 0) {
              sum += v * v;
            }
          }

          std::cout << "Somme des carres pairs = " << sum << '\n';
          return 0;
        }
        ```
      ],
      [
        ```rs
        fn main() {
            let values = vec![1, 2, 3, 4, 5, 6];

            let sum: i32 = values
                .iter()
                .filter(|&&v| v % 2 == 0)
                .map(|&v| v * v)
                .sum();

            println!("Somme des carres pairs = {}", sum);
        }
        ```
      ],
    ),
    caption: [Différence de syntaxe entre C++ et Rust],
    supplement: "Code",
  )

  == Bibliothèques (_Crates_)

  Ce projet utilise la bibliothèque graphique egui@egui, qui a pour avantage d'avoir un design moderne avec relativement peu de code (notamment comparé à d'autres bibliothèques comme GTK ou Qt).

  Les autres bibliothèques utilisées sont principalement des utilitaires :
  - `rand` : permet de générer des nombres aléatoires
  - `inventory` : rend les listes d'algorithmes implémentés plus facile à récupérer et à afficher dans l'interface graphiques

  == Choix des méthodes

  Vu les méthodes vues en cours, j'ai d'abord choisi d'implémenter un recuit simulé : il fallait une méthode à base de voisinage, et c'est une méthode qui converge rapidement vers un optimum local, tout en évitant les faux optima de la méthode de descente.

  Ensuite j'ai voulu comparer avec une méthode qui n'était pas à base de voisinage. J'avais déjà utilisé les algorithmes génétiques dans d'autres projets, et j'étais curieux de comprendre en pratique comment l'algorithme de colonies de fourmis fonctionnait. J'ai donc pris cette option, ce qui me permettait de pouvoir également comparer les algorithmes à base de population.

  J'ai enfin implémenter une méthode de descente, pour pouvoir comparer les algorithmes à base de voisinage avec une méthode plus simple.

  = Résultats

  == Application graphique

  J'ai créé l'interface graphique en premier (avant même d'implémenter les algorithmes), pour pouvoir visualiser les résultats de mes algorithmes au fur et à mesure de leur implémentation. L'interface est relativement simple : sur la gauche, un panel permet de régler tous les paramètres de l'instance et des algorithmes, tandis que sur la droite, une visualisation de l'instance et des résultats est affichée.

  #rounded-image(image("assets/menu-ux.png"))

  Pour faire fonctionner un algorithme, il faut d'abord charger un fichier d'instance (au format `.vrptw`). Le programme scanne le dossier `data` à partir de l'endroit où le projet est lancé (s'il est lancé avec `Cargo`, alors le dossier devrait être au bon endroit). Ensuite, il faut charger une solution valide initiale. Trois choix sont possibles :

  - Le plus simple : Chaque client est livré par un camion différent. On obtient alors ce genre de graphe "étoilé"

  #rounded-image(image("assets/simplest.png"))

  - Algorithme glouton : On mélange les clients, puis on fait tourner un camion : tant que le camion a assez de capacité, on utilise le même camion. On recommence ensuite avec un nouveau camion, jusqu'à ce que tous les clients soient livrés. On obtient alors un graphe beaucoup moins organisé :

  #rounded-image(image("assets/greedy.png"))

  - Algorithme aléatoire : Sur le même principe que l'algorithme glouton, mais une part d'aléatoire s'ajoute : même si le camion a assez de capacité pour livrer un client, il a une chance sur dix de ne pas accecpter le client, et donc on passe au camion suivant. Cette solution ne donne pas vraiment de résultats meilleurs que l'algorithme glouton cependant.

  #block-full(title: "Avertissement important", stroke-color: red)[
    Quand le temps est pris en compte, seule la solution la plus simple crée une solution valide. Les deux autres solutions ne garantissent pas que les fenêtres de temps soient respectées.

    En effet, si l'on ajoute la dimension de temps simplement à cet algorithme, on se retrouve quasiment dans le cas le plus simple (rare sont les situations où un même camion peut desservir plus de 2 ou 3 clients dans la solution initiale).
  ]

  Une fois la solution initiale chargée, le bouton `Résoudre` devrait redevenir cliquable, et il lancera l'algorithme présent dans la liste déroulante juste en dessous. Les paramètres doivent être ajustés avant de cliquer sur le bouton.

  Pour la résolution deux choix sont possibles, et représentés par les cases en haut :

  - Prise en compte du temps : les algorithmes fonctionnent avec et sans la dimension de temps, il est possible de l'activer ou de la désactiver.

  - Affichage des étapes : afficher les situations intermédiaires rend l'algorithme plus passionant, mais son affichage fausse les temps de résolution. Ainsi il peut être judicieux de désactiver l'affichage des étapes pour avoir une meilleure idée du temps de résolution.

  == Résultats obtenus (tests)

  #let tests(filename, caption: none, height: 16%) = {
    let files = (101, 102, 111, 112, 201, 202, 1101, 1102, 1201, 1202).map(f => (
      "../plots/" + filename + "data" + str(f) + ".png"
    ))
    figure(
      grid(
        columns: 2,
        gutter: 10pt,
        ..files.map(f => image(f, height: height)),
      ),
      caption: caption,
      kind: "Comparaison",
      supplement: "Comparaison",
    )
  }

  === Tests paramétriques

  Dans un premier temps, j'ai voulu tester la sensibilité de chacun des algorithmes significatifs (SA, ACO) à leurs paramètres respectifs.

  #block-left(title: "Note")[
    Tous les graphes présentés dans cette section sont disponibles dans les annexes.
  ]

  ==== Recuit simulé

  #let python_json = json("../python_stats/outputs/best_fit_summary.json")

  #let math_eval(equation) = eval(equation, mode: "math")

  ===== Facteur de refroissement $alpha$

  #tests(
    "sa_alpha_",
    caption: [Distance en fonction du facteur de refroidissement $alpha$ pour chacun des fichiers proposés],
  )

  On observe clairement une amélioration de la solution à mesure que le facteur de refroidissement augmente. Ces graphes ne le montrent pas, mais évidemment le temps de traitement augmente également. On pouvait s'attendre à ce résultat, puisque si le facteur de refroissement augmente, la température diminue plus lentement, et donc l'algorithme fait plus d'itérations.

  Le plus intéressant serait de savoir quel est la courbe qui collerait le plus à ces valeurs. Avec un petit script python, on peut voir quel modèle (quadratique, logarithmique, linéaire, cubique, exponentiel, ...) serait le plus adapté. Pour les deux premiers fichiers, c'est une courbe quadratique qui semble la plus adaptée (avec des formules quelque peu étonnantes#footnote[La formule pour le premier graphe est : #math_eval(python_json.at(0).equation.replace("x", "alpha").replace("y", "\"score\""))]). Pour les suivants, il y a un combat entre les courbes de type _shifting power gap_ et les courbes cubiques. Dans tous les cas, ce n'est jamais une courbe linéaire. Ainsi l'influence du facteur de refroidissement est plus importante à mesure que celui-ci augmente, et ce au moins de l'ordre du carré.

  #pagebreak()

  ===== Température initiale $T_0$

  #tests(
    "Temp - ",
    caption: [Distance en fonction de la température initiale $T_0$ pour chacun des fichiers proposés],
  )

  Pour la température initiale, on voit des résultats beaucoup plus mitigés. Autant pour certains fichiers comme le premier, le deuxième, le septième et le huitième, on semble avoir une influence positive sur le résultat (la courbe semble, malgré beaucoup de bruit, être décroissante), autant pour les autres fichiers, en dehors d'une baisse significative pour une température initialé inférieure à 20°, il n'y a pas de tendance claire. En tout cas, l'influence est largement moins visible que pour $alpha$.

  #pagebreak()

  ===== Température finale $T_f$

  #tests(
    "sa_t_final_",
    caption: [Distance en fonction de la température finale $T_f$ pour chacun des fichiers proposés],
  )

  Pour la témpérature finale en revanche, l'influence est beaucoup plus nette : Pour tous les graphes, quand la température finale se trouve entre 1 et 10 degrés, on obtient une fonction linéaire croissante. Ainsi, jusqu'à environ 1 degré, il est toujours intéressant (en tout cas au vu du panel de solutions testées) de faire baisser la température finale, puisque son influence ne s'amenuise pas avec le temps.

  En revanche, en dessous de 1 degré, l'influence de la température finale est moins claire : pour tous les fichiers, il semble y avoir une irrégularité dans les courbes,


  = Utilisation de l'IA dans ce projet

  L'intelligence artificielle a été utilisée dans ce projet. Pour avoir un code fiable et opérationnel, j'ai surtout utilisée les deux IA suivantes :

  - Claude Sonnet 4.6 Thinking
  - ChatGPT 5.3 Codex

  Elles avaient deux missions différentes : Claude m'aidait majoritairement dans le design du code, tandis que ChatGPT m'aidait à régler les problèmes subsidiaires de syntaxe ou de logique que je pouvais avoir.

  Dans un premier temps, j'ai demandé à Claude Sonnet de créer le projet dans son intégralité. Évidemment, certaines erreurs étaient présentes (malgré des résultats franchement surprenants), et j'ai décidé par la suite de repartir de zéro, tout en gardant la strcture générale du code qu'il avait créé.

  Ma seconde utilisation de l'IA a été de créer les tests. Ceux-ci sont particulièrement pénibles à écrire, donc j'ai demandé à ChatGPT de les implémenter pour moi, en gardant la main sur l'idée des tests que je voulais réaliser.

  #bibliography("bib.yaml", title: "Bibliographie")
]
