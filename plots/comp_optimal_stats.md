# Benchmark VRPTW - Paramètres Optimaux
Benchmark utilisant les paramètres optimaux trouvés dans le rapport.

## Paramètres utilisés

### Simulated Annealing

- t_initial: 500.0
- t_final: 5.0 (optimal entre 1-10 selon rapport)
- alpha: 0.9995 (plus élevé = mieux)
- iter_per_temp: 75 (optimal entre 50-100)

### Ant Colony Optimization

- n_ants: 50 (plus augmente = mieux)
- alpha: 1.75 (optimal entre 1.5-2)
- beta: 2.5 (pas de recommandation spécifique)
- rho: 0.03 (optimal < 0.05)
- q: 400.0 (optimal entre 300-500)
- max_iterations: 750

## Résultats

- Instances: 10
- Runs par instance et par algorithme: 10

| Instance | Algorithme | Dist. moyenne | Écart-type | Dist. min | Temps moyen (ms) | Faisabilité | Nb routes moyen | Gap normalisé (%) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| data101.vrp | Simulated Annealing (Optimal) | 1859.680 | 66.857 | 1788.021 | 13625.00 | 100.0% | 20.80 | 0.54 |
| data101.vrp | Ant Colony Optimization (Optimal) | 1979.756 | 37.013 | 1924.227 | 32810.75 | 0.0% | 27.90 | 7.03 |
| data101.vrp | Hill Climbing | 1849.766 | 64.191 | 1780.073 | 1663.15 | 100.0% | 23.10 | 0.00 |
| data102.vrp | Simulated Annealing (Optimal) | 1768.176 | 51.063 | 1709.362 | 12479.60 | 100.0% | 18.70 | 4.86 |
| data102.vrp | Ant Colony Optimization (Optimal) | 1801.187 | 23.667 | 1774.745 | 29612.72 | 0.0% | 24.00 | 6.81 |
| data102.vrp | Hill Climbing | 1686.297 | 38.524 | 1625.007 | 1802.01 | 100.0% | 21.20 | 0.00 |
| data1101.vrp | Simulated Annealing (Optimal) | 1881.111 | 65.361 | 1792.682 | 12314.18 | 100.0% | 17.60 | 0.00 |
| data1101.vrp | Ant Colony Optimization (Optimal) | 2122.966 | 33.358 | 2087.841 | 29924.71 | 0.0% | 22.80 | 12.86 |
| data1101.vrp | Hill Climbing | 1976.107 | 72.238 | 1859.461 | 1647.75 | 100.0% | 20.20 | 5.05 |
| data1102.vrp | Simulated Annealing (Optimal) | 1756.013 | 44.997 | 1648.605 | 12063.24 | 100.0% | 15.90 | 0.00 |
| data1102.vrp | Ant Colony Optimization (Optimal) | 1956.596 | 40.962 | 1870.154 | 28323.19 | 0.0% | 20.20 | 11.42 |
| data1102.vrp | Hill Climbing | 1827.133 | 76.766 | 1703.739 | 1563.33 | 100.0% | 18.70 | 4.05 |
| data111.vrp | Simulated Annealing (Optimal) | 1435.527 | 47.036 | 1381.857 | 11745.89 | 100.0% | 14.00 | 15.02 |
| data111.vrp | Ant Colony Optimization (Optimal) | 1378.779 | 30.563 | 1315.531 | 26320.98 | 0.0% | 15.40 | 10.47 |
| data111.vrp | Hill Climbing | 1248.097 | 25.772 | 1201.250 | 1551.73 | 100.0% | 14.70 | 0.00 |
| data112.vrp | Simulated Annealing (Optimal) | 1400.767 | 37.102 | 1339.319 | 11640.94 | 100.0% | 13.10 | 24.63 |
| data112.vrp | Ant Colony Optimization (Optimal) | 1123.938 | 24.114 | 1082.232 | 24811.33 | 0.0% | 11.60 | 0.00 |
| data112.vrp | Hill Climbing | 1159.776 | 39.905 | 1095.399 | 1408.80 | 100.0% | 13.50 | 3.19 |
| data1201.vrp | Simulated Annealing (Optimal) | 1568.891 | 68.636 | 1489.048 | 9642.95 | 100.0% | 7.60 | 0.00 |
| data1201.vrp | Ant Colony Optimization (Optimal) | 1812.576 | 54.443 | 1714.484 | 27634.01 | 0.0% | 12.30 | 15.53 |
| data1201.vrp | Hill Climbing | 1607.011 | 42.067 | 1551.363 | 1252.10 | 100.0% | 13.70 | 2.43 |
| data1202.vrp | Simulated Annealing (Optimal) | 1440.901 | 62.889 | 1303.552 | 9434.36 | 100.0% | 6.30 | 0.00 |
| data1202.vrp | Ant Colony Optimization (Optimal) | 1626.824 | 32.136 | 1574.063 | 26372.78 | 0.0% | 10.30 | 12.90 |
| data1202.vrp | Hill Climbing | 1474.122 | 94.697 | 1303.190 | 1306.57 | 100.0% | 12.70 | 2.31 |
| data201.vrp | Simulated Annealing (Optimal) | 1525.317 | 35.773 | 1471.200 | 9451.60 | 100.0% | 7.30 | 7.20 |
| data201.vrp | Ant Colony Optimization (Optimal) | 1553.704 | 23.235 | 1509.164 | 28322.15 | 0.0% | 12.00 | 9.20 |
| data201.vrp | Hill Climbing | 1422.846 | 52.426 | 1318.664 | 1256.80 | 100.0% | 14.80 | 0.00 |
| data202.vrp | Simulated Annealing (Optimal) | 1408.744 | 33.311 | 1355.805 | 9408.79 | 100.0% | 6.80 | 11.37 |
| data202.vrp | Ant Colony Optimization (Optimal) | 1365.826 | 35.595 | 1318.234 | 27085.42 | 0.0% | 10.00 | 7.98 |
| data202.vrp | Hill Climbing | 1264.910 | 34.257 | 1228.085 | 1560.93 | 100.0% | 13.60 | 0.00 |
