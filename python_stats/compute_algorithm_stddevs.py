#!/usr/bin/env python3
"""
Calcule les ecarts-types par algorithme a partir d'un CSV de runs,
puis la moyenne de ces ecarts-types par algorithme.

Le CSV d'entree doit contenir les colonnes:
- instance_index
- algorithm
- run_number
- distance
- time_ms
"""

from __future__ import annotations

import argparse
import csv
import math
import statistics
from collections import defaultdict
from pathlib import Path
from typing import DefaultDict, Dict, List, Tuple


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Calcule ecarts-types (distance/time) par algorithme et moyenne par algorithme.",
    )
    parser.add_argument(
        "--csv",
        type=Path,
        default=Path("plots/comp_optimal_detailed.csv"),
        help="Fichier CSV source (defaut: plots/comp_optimal_detailed.csv)",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=Path("python_stats/outputs/comp_optimal_stddev_summary.csv"),
        help="CSV de sortie pour le resume (defaut: python_stats/outputs/comp_optimal_stddev_summary.csv)",
    )
    return parser.parse_args()


def safe_stdev(values: List[float]) -> float:
    if len(values) < 2:
        return float("nan")
    return statistics.stdev(values)


def read_rows(csv_path: Path) -> List[Dict[str, str]]:
    with csv_path.open("r", newline="", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        required = {"instance_index", "algorithm", "run_number", "distance", "time_ms"}
        if reader.fieldnames is None:
            raise ValueError(f"Aucun en-tete detecte dans {csv_path}")

        missing = required.difference(reader.fieldnames)
        if missing:
            raise ValueError(
                f"Colonnes manquantes dans {csv_path}: {sorted(missing)}; colonnes presentes: {reader.fieldnames}"
            )

        rows: List[Dict[str, str]] = []
        for row in reader:
            rows.append(row)
    return rows


def compute_stats(rows: List[Dict[str, str]]) -> Tuple[List[Dict[str, float]], List[Dict[str, float]]]:
    grouped: DefaultDict[Tuple[str, int], Dict[str, List[float]]] = defaultdict(
        lambda: {"distance": [], "time_ms": []}
    )

    for row in rows:
        algo = row["algorithm"].strip()
        instance = int(row["instance_index"])
        distance = float(row["distance"])
        time_ms = float(row["time_ms"])

        key = (algo, instance)
        grouped[key]["distance"].append(distance)
        grouped[key]["time_ms"].append(time_ms)

    detailed: List[Dict[str, float]] = []
    for (algo, instance), values in sorted(grouped.items(), key=lambda x: (x[0][0], x[0][1])):
        std_distance = safe_stdev(values["distance"])
        std_time = safe_stdev(values["time_ms"])
        detailed.append(
            {
                "algorithm": algo,
                "instance_index": float(instance),
                "std_distance": std_distance,
                "std_time_ms": std_time,
            }
        )

    by_algo: DefaultDict[str, Dict[str, List[float]]] = defaultdict(
        lambda: {"std_distance": [], "std_time_ms": []}
    )
    for row in detailed:
        algo = str(row["algorithm"])
        if not math.isnan(row["std_distance"]):
            by_algo[algo]["std_distance"].append(float(row["std_distance"]))
        if not math.isnan(row["std_time_ms"]):
            by_algo[algo]["std_time_ms"].append(float(row["std_time_ms"]))

    summary: List[Dict[str, float]] = []
    for algo in sorted(by_algo.keys()):
        distance_stds = by_algo[algo]["std_distance"]
        time_stds = by_algo[algo]["std_time_ms"]
        summary.append(
            {
                "algorithm": algo,
                "mean_std_distance": statistics.mean(distance_stds) if distance_stds else float("nan"),
                "mean_std_time_ms": statistics.mean(time_stds) if time_stds else float("nan"),
                "n_instances": float(len(distance_stds)),
            }
        )

    return detailed, summary


def write_summary_csv(summary_rows: List[Dict[str, float]], output_path: Path) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(
            f,
            fieldnames=["algorithm", "mean_std_distance", "mean_std_time_ms", "n_instances"],
        )
        writer.writeheader()
        writer.writerows(summary_rows)


def print_results(detailed_rows: List[Dict[str, float]], summary_rows: List[Dict[str, float]]) -> None:
    print("=== Ecart-type par algorithme et par instance ===")
    print("algorithm,instance_index,std_distance,std_time_ms")
    for row in detailed_rows:
        print(
            f"{row['algorithm']},{int(row['instance_index'])},{row['std_distance']:.6f},{row['std_time_ms']:.6f}"
        )

    print()
    print("=== Moyenne des ecarts-types par algorithme ===")
    print("algorithm,mean_std_distance,mean_std_time_ms,n_instances")
    for row in summary_rows:
        print(
            f"{row['algorithm']},{row['mean_std_distance']:.6f},{row['mean_std_time_ms']:.6f},{int(row['n_instances'])}"
        )


def main() -> None:
    args = parse_args()
    rows = read_rows(args.csv)
    detailed, summary = compute_stats(rows)
    write_summary_csv(summary, args.output)
    print_results(detailed, summary)
    print()
    print(f"Resume ecrit dans: {args.output}")


if __name__ == "__main__":
    main()
