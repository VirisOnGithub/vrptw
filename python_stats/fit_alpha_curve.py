#!/usr/bin/env python3
"""
Fit the best curve for alpha -> distance CSV files.

Example:
    python3 python_stats/fit_alpha_curve.py --csv plots/sa_alpha_data101.csv --plot
    python3 python_stats/fit_alpha_curve.py --glob "plots/sa_alpha_*.csv" --plot
"""

from __future__ import annotations

import argparse
import csv
import glob
import json
import math
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Iterable, List, Sequence, Tuple


EPS = 1e-12


@dataclass
class FitResult:
    name: str
    equation: str
    y_hat: List[float]
    params: dict
    k: int
    rmse: float
    r2: float
    adj_r2: float
    aic: float
    aicc: float


def read_points(csv_path: Path, x_col: str, y_col: str) -> Tuple[List[float], List[float]]:
    xs: List[float] = []
    ys: List[float] = []

    with csv_path.open("r", newline="", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        if reader.fieldnames is None:
            raise ValueError(f"No header found in {csv_path}")
        missing = [c for c in (x_col, y_col) if c not in reader.fieldnames]
        if missing:
            raise ValueError(
                f"Missing columns {missing} in {csv_path}. Available: {reader.fieldnames}"
            )

        for row in reader:
            x_raw = row.get(x_col, "").strip()
            y_raw = row.get(y_col, "").strip()
            if not x_raw or not y_raw:
                continue
            try:
                x = float(x_raw)
                y = float(y_raw)
            except ValueError:
                continue
            if math.isnan(x) or math.isnan(y) or math.isinf(x) or math.isinf(y):
                continue
            xs.append(x)
            ys.append(y)

    if len(xs) < 4:
        raise ValueError(f"Not enough valid points in {csv_path} (need >= 4)")

    points = sorted(zip(xs, ys), key=lambda p: p[0])
    xs_sorted = [p[0] for p in points]
    ys_sorted = [p[1] for p in points]
    return xs_sorted, ys_sorted


def solve_linear_system(a: List[List[float]], b: List[float]) -> List[float]:
    n = len(b)
    aug = [row[:] + [rhs] for row, rhs in zip(a, b)]

    for col in range(n):
        pivot_row = max(range(col, n), key=lambda r: abs(aug[r][col]))
        if abs(aug[pivot_row][col]) < EPS:
            raise ValueError("Singular matrix while solving normal equations")
        if pivot_row != col:
            aug[col], aug[pivot_row] = aug[pivot_row], aug[col]

        pivot = aug[col][col]
        for j in range(col, n + 1):
            aug[col][j] /= pivot

        for r in range(n):
            if r == col:
                continue
            factor = aug[r][col]
            if abs(factor) < EPS:
                continue
            for j in range(col, n + 1):
                aug[r][j] -= factor * aug[col][j]

    return [aug[i][n] for i in range(n)]


def least_squares_linear(
    xs: Sequence[float], ys: Sequence[float], basis: Sequence[Callable[[float], float]]
) -> List[float]:
    k = len(basis)
    xtx = [[0.0 for _ in range(k)] for _ in range(k)]
    xty = [0.0 for _ in range(k)]

    for x, y in zip(xs, ys):
        phi = [f(x) for f in basis]
        for i in range(k):
            xty[i] += phi[i] * y
            for j in range(k):
                xtx[i][j] += phi[i] * phi[j]

    return solve_linear_system(xtx, xty)


def compute_metrics(ys: Sequence[float], y_hat: Sequence[float], k: int) -> Tuple[float, float, float, float, float]:
    n = len(ys)
    sse = sum((y - yh) ** 2 for y, yh in zip(ys, y_hat))
    mse = sse / max(n, 1)
    rmse = math.sqrt(mse)

    y_mean = sum(ys) / n
    sst = sum((y - y_mean) ** 2 for y in ys)
    r2 = 1.0 - (sse / sst) if sst > EPS else float("nan")

    if n - k - 1 > 0 and not math.isnan(r2):
        adj_r2 = 1.0 - (1.0 - r2) * (n - 1) / (n - k - 1)
    else:
        adj_r2 = float("nan")

    sse_for_log = max(sse, EPS)
    aic = n * math.log(sse_for_log / n) + 2 * k
    if n - k - 1 > 0:
        aicc = aic + (2 * k * (k + 1)) / (n - k - 1)
    else:
        aicc = float("inf")

    return rmse, r2, adj_r2, aic, aicc


def fit_linear_model(
    xs: Sequence[float],
    ys: Sequence[float],
    name: str,
    basis: Sequence[Callable[[float], float]],
    equation_builder: Callable[[Sequence[float]], str],
) -> FitResult:
    coeffs = least_squares_linear(xs, ys, basis)
    y_hat = []
    for x in xs:
        y = 0.0
        for c, f in zip(coeffs, basis):
            y += c * f(x)
        y_hat.append(y)

    rmse, r2, adj_r2, aic, aicc = compute_metrics(ys, y_hat, len(coeffs))
    params = {f"p{i}": coeffs[i] for i in range(len(coeffs))}

    return FitResult(
        name=name,
        equation=equation_builder(coeffs),
        y_hat=y_hat,
        params=params,
        k=len(coeffs),
        rmse=rmse,
        r2=r2,
        adj_r2=adj_r2,
        aic=aic,
        aicc=aicc,
    )


def fit_shifted_exp_model(
    xs: Sequence[float],
    ys: Sequence[float],
    name: str,
    transform_u: Callable[[float], float],
    transform_label: str,
    c_steps: int = 1200,
) -> FitResult | None:
    y_min = min(ys)
    y_max = max(ys)
    span = max(y_max - y_min, 1.0)

    c_lo = y_min - 4.0 * span
    c_hi = y_min - 1e-9
    if c_hi <= c_lo:
        return None

    best: FitResult | None = None

    for i in range(c_steps):
        c = c_lo + (c_hi - c_lo) * (i / max(c_steps - 1, 1))

        try:
            u_values = [transform_u(x) for x in xs]
        except (ValueError, ZeroDivisionError):
            return None

        z_values = []
        valid = True
        for y in ys:
            shifted = y - c
            if shifted <= 0:
                valid = False
                break
            z_values.append(math.log(shifted))
        if not valid:
            continue

        try:
            a0, b0 = least_squares_linear(u_values, z_values, [lambda t: 1.0, lambda t: t])
        except ValueError:
            continue

        y_hat = [c + math.exp(a0 + b0 * u) for u in u_values]
        rmse, r2, adj_r2, aic, aicc = compute_metrics(ys, y_hat, 3)

        fit = FitResult(
            name=name,
            equation=f"y = c + exp(a + b*{transform_label})",
            y_hat=y_hat,
            params={"a": a0, "b": b0, "c": c},
            k=3,
            rmse=rmse,
            r2=r2,
            adj_r2=adj_r2,
            aic=aic,
            aicc=aicc,
        )

        if best is None or fit.aicc < best.aicc:
            best = fit

    return best


def fit_all_models(xs: Sequence[float], ys: Sequence[float]) -> List[FitResult]:
    models: List[FitResult] = []

    models.append(
        fit_linear_model(
            xs,
            ys,
            name="linear",
            basis=[lambda x: 1.0, lambda x: x],
            equation_builder=lambda p: f"y = {p[0]:.6g} + {p[1]:.6g}*x",
        )
    )

    models.append(
        fit_linear_model(
            xs,
            ys,
            name="quadratic",
            basis=[lambda x: 1.0, lambda x: x, lambda x: x * x],
            equation_builder=lambda p: f"y = {p[0]:.6g} + {p[1]:.6g}*x + {p[2]:.6g}*x^2",
        )
    )

    models.append(
        fit_linear_model(
            xs,
            ys,
            name="cubic",
            basis=[lambda x: 1.0, lambda x: x, lambda x: x * x, lambda x: x * x * x],
            equation_builder=lambda p: (
                f"y = {p[0]:.6g} + {p[1]:.6g}*x + {p[2]:.6g}*x^2 + {p[3]:.6g}*x^3"
            ),
        )
    )

    if all(1.0 - x > EPS for x in xs):
        models.append(
            fit_linear_model(
                xs,
                ys,
                name="log_gap",
                basis=[lambda x: 1.0, lambda x: math.log(1.0 - x)],
                equation_builder=lambda p: f"y = {p[0]:.6g} + {p[1]:.6g}*ln(1-x)",
            )
        )

        models.append(
            fit_linear_model(
                xs,
                ys,
                name="inverse_gap",
                basis=[lambda x: 1.0, lambda x: 1.0 / (1.0 - x)],
                equation_builder=lambda p: f"y = {p[0]:.6g} + {p[1]:.6g}/(1-x)",
            )
        )

        shifted_power = fit_shifted_exp_model(
            xs,
            ys,
            name="shifted_power_gap",
            transform_u=lambda x: math.log(1.0 - x),
            transform_label="ln(1-x)",
        )
        if shifted_power is not None:
            models.append(shifted_power)

    shifted_exp_x = fit_shifted_exp_model(
        xs,
        ys,
        name="shifted_exp_x",
        transform_u=lambda x: x,
        transform_label="x",
    )
    if shifted_exp_x is not None:
        models.append(shifted_exp_x)

    return models


def pick_best(models: Sequence[FitResult]) -> FitResult:
    return min(models, key=lambda m: (m.aicc, m.rmse))


def predict_on_grid(model: FitResult, x_grid: Sequence[float]) -> List[float]:
    if model.name == "linear":
        p0 = model.params["p0"]
        p1 = model.params["p1"]
        return [p0 + p1 * x for x in x_grid]

    if model.name == "quadratic":
        p0 = model.params["p0"]
        p1 = model.params["p1"]
        p2 = model.params["p2"]
        return [p0 + p1 * x + p2 * x * x for x in x_grid]

    if model.name == "cubic":
        p0 = model.params["p0"]
        p1 = model.params["p1"]
        p2 = model.params["p2"]
        p3 = model.params["p3"]
        return [p0 + p1 * x + p2 * x * x + p3 * x * x * x for x in x_grid]

    if model.name == "log_gap":
        p0 = model.params["p0"]
        p1 = model.params["p1"]
        return [p0 + p1 * math.log(1.0 - x) for x in x_grid]

    if model.name == "inverse_gap":
        p0 = model.params["p0"]
        p1 = model.params["p1"]
        return [p0 + p1 / (1.0 - x) for x in x_grid]

    if model.name == "shifted_power_gap":
        a = model.params["a"]
        b = model.params["b"]
        c = model.params["c"]
        return [c + math.exp(a + b * math.log(1.0 - x)) for x in x_grid]

    if model.name == "shifted_exp_x":
        a = model.params["a"]
        b = model.params["b"]
        c = model.params["c"]
        return [c + math.exp(a + b * x) for x in x_grid]

    raise ValueError(f"Unknown model {model.name}")


def save_plot(
    csv_path: Path,
    xs: Sequence[float],
    ys: Sequence[float],
    best: FitResult,
    output_dir: Path,
) -> Path | None:
    try:
        import matplotlib.pyplot as plt
    except Exception:
        return None

    x_min, x_max = min(xs), max(xs)
    if abs(x_max - x_min) < EPS:
        x_grid = [x_min]
    else:
        steps = 300
        x_grid = [x_min + (x_max - x_min) * i / (steps - 1) for i in range(steps)]

    y_grid = predict_on_grid(best, x_grid)

    output_dir.mkdir(parents=True, exist_ok=True)
    out_path = output_dir / f"{csv_path.stem}_best_fit.png"

    fig, ax = plt.subplots(figsize=(8, 5))
    ax.scatter(xs, ys, label="Data", color="#1f77b4", s=45)
    ax.plot(x_grid, y_grid, label=f"Best: {best.name}", color="#d62728", linewidth=2)
    ax.set_title(f"Best fit for {csv_path.name}")
    ax.set_xlabel("alpha")
    ax.set_ylabel("mean_distance")
    ax.grid(alpha=0.25)
    ax.legend()
    fig.tight_layout()
    fig.savefig(out_path, dpi=160)
    plt.close(fig)
    return out_path


def save_report(csv_path: Path, models: Sequence[FitResult], best: FitResult, output_dir: Path) -> Path:
    output_dir.mkdir(parents=True, exist_ok=True)
    out_path = output_dir / f"{csv_path.stem}_best_fit.json"

    payload = {
        "csv": str(csv_path),
        "best_model": {
            "name": best.name,
            "equation": best.equation,
            "params": best.params,
            "metrics": {
                "rmse": best.rmse,
                "r2": best.r2,
                "adj_r2": best.adj_r2,
                "aic": best.aic,
                "aicc": best.aicc,
            },
        },
        "all_models": [
            {
                "name": m.name,
                "equation": m.equation,
                "params": m.params,
                "metrics": {
                    "rmse": m.rmse,
                    "r2": m.r2,
                    "adj_r2": m.adj_r2,
                    "aic": m.aic,
                    "aicc": m.aicc,
                },
            }
            for m in sorted(models, key=lambda x: x.aicc)
        ],
    }

    out_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    return out_path


def process_file(csv_path: Path, args: argparse.Namespace) -> dict:
    xs, ys = read_points(csv_path, args.x_col, args.y_col)
    models = fit_all_models(xs, ys)
    best = pick_best(models)

    report_path = save_report(csv_path, models, best, args.output_dir)
    plot_path = save_plot(csv_path, xs, ys, best, args.output_dir) if args.plot else None

    return {
        "csv": str(csv_path),
        "best_model": best.name,
        "equation": best.equation,
        "rmse": best.rmse,
        "r2": best.r2,
        "aicc": best.aicc,
        "report": str(report_path),
        "plot": str(plot_path) if plot_path else None,
    }


def resolve_input_files(args: argparse.Namespace) -> List[Path]:
    files: List[Path] = []
    if args.csv:
        files.append(Path(args.csv))
    if args.glob:
        files.extend(Path(p) for p in glob.glob(args.glob))

    # Keep deterministic order and remove duplicates.
    unique = sorted({p.resolve() for p in files})
    if not unique:
        raise ValueError("No input file found. Use --csv or --glob.")

    return [Path(p) for p in unique]


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Find the best curve for alpha -> distance data in CSV files"
    )
    parser.add_argument("--csv", type=str, help="Path to one CSV file")
    parser.add_argument("--glob", type=str, help="Glob pattern for multiple CSV files")
    parser.add_argument("--x-col", type=str, default="alpha", help="X column name")
    parser.add_argument(
        "--y-col", type=str, default="mean_distance", help="Y column name"
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=Path("python_stats") / "outputs",
        help="Directory for reports and plots",
    )
    parser.add_argument(
        "--plot",
        action="store_true",
        help="Generate PNG plots if matplotlib is available",
    )
    return parser


def main() -> None:
    parser = build_parser()
    args = parser.parse_args()

    files = resolve_input_files(args)

    print(f"Found {len(files)} file(s) to process")
    results = []
    for csv_path in files:
        result = process_file(csv_path, args)
        results.append(result)

        print("-" * 80)
        print(f"File      : {result['csv']}")
        print(f"Best model: {result['best_model']}")
        print(f"Equation  : {result['equation']}")
        print(f"RMSE      : {result['rmse']:.6f}")
        print(f"R2        : {result['r2']:.6f}")
        print(f"AICc      : {result['aicc']:.6f}")
        print(f"Report    : {result['report']}")
        if result["plot"]:
            print(f"Plot      : {result['plot']}")

    summary_path = args.output_dir / "best_fit_summary.json"
    args.output_dir.mkdir(parents=True, exist_ok=True)
    summary_path.write_text(json.dumps(results, indent=2), encoding="utf-8")
    print("-" * 80)
    print(f"Summary   : {summary_path}")


if __name__ == "__main__":
    main()
