Projet – Vehicle Routing Problem with Time Windows

Objectif :
L’objectif est de trouver des solutions au problème du VRPTW en utilisant 2 métaheuristiques parmi
les méthodes vues en cours.
Le VRPTW consiste à déterminer un ensemble d’itinéraires,
commençant et se terminant au dépôt, qui couvrent un ensemble
de clients. Chaque client a une demande spécifique et est visité
une seule fois et par un seul véhicule. Tous les véhicules ont la
même capacité C et transportent un seul type de marchandises.
Aucun véhicule ne peut desservir plus de clients que sa capacité C
ne le permet. L’objectif ici est de réduire au minimum la distance
totale parcourue par l’ensemble des véhicules (le nombre de
véhicules utilisés est à déterminer, il n’est pas limité).
Il est conseillé, mais pas obligatoire, de faire une visualisation des tournées afin de pouvoir vérifier la
validité de vos opérateurs de voisinages et de vos résultats.
Vous devez :
1. Modéliser ce problème et mettre en place la structure de votre code.
2. Pour chaque jeu de données, déterminer le nombre minimum de véhicules à utiliser.
Faire le projet tout d’abord sans tenir compte des fenêtres de temps puis en les prenant en compte.
3. Créer un générateur aléatoire de solutions.
4. Implémenter 2 métaheuristiques et les tester sur les fichiers de données téléchargeables sur
MOODLE avec un protocole de tests clairement expliqué ainsi qu’une analyse des résultats.
Chaque fichier contient une liste de clients (avec ses coordonnées euclidiennes et la quantité
d’articles demandés). Le client avec le numéro 0 correspond au dépôt.
5. Comparer les deux algorithmes en termes de temps d’exécution, de qualité des solutions
obtenues, de nombre de solutions générées en fonction des structures de voisinages utilisées
et des valeurs des paramètres. Discuter les résultats.
6. Bonus : en utilisant un package de programmation linéaire, essayer de déterminer à partir de
quelle limite (nombre de clients par exemple) il devient difficile d’obtenir une solution (utiliser
la modélisation donnée en cours). Pour cela construisez vous-même des jeux de données de
plus en gros à partir d’un des jeux de données.
Vous devez fournir un rapport en PDF expliquant et illustrant l’ensemble du travail réalisé, et fournir
le code associé (en indiquant comment l’exécuter). Tout ceci devra être déposé dans un ZIP à votre
nom dans la « Zone de dépôt » du module Moodle associé au cours.