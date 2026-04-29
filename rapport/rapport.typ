#import "@local/polytech:1.0.0": *;

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
      gutter: 40pt,
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
  

  #bibliography("bib.yaml", title: "Bibliographie")
]
