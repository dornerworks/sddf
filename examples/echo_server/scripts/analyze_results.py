# Copyright 2026, DornerWorks
# SPDX-License-Identifier: BSD-2-Clause

# --- Standard Library ---
import argparse
from enum import Enum
import glob
from io import StringIO
import json
import os
import subprocess
import sys
import warnings
# --- Third Party ---
import pandas as pd
import plotly.colors as pc
from plotly.subplots import make_subplots
import plotly.graph_objects as go

# --- Globals ---
plot_cfg = {
    "height": 800,
    "width": 1250,
    "colors": pc.qualitative.Plotly,
    "marker_size": 6,
}

# --- Classes ---
class UtilizationType(Enum):
    SYSTEM = 1
    KERNEL = 2
    USER   = 3

# --- Functions ---
def parse_args():
    parser = argparse.ArgumentParser(
        description="A script for processing benchmarking results produced "
                    "using the echo_server example of the sddf repository."
    )
    parser.add_argument(
        "-i", "--input",
        required=True,
        type=str,
        help="The path to the results directory."
    )
    args = parser.parse_args()

    return args

def process_serial_logs(
        path: str
):
    print("Processing serial logs")

    # noinspection bad-argument-type
    py_file = os.path.join(
        os.path.dirname(__file__),
        "process_output.py"
    )

    search_pattern = os.path.join(path, "*.log")
    paths = glob.glob(search_pattern)
    for p in paths:
        out_path = f"{os.path.splitext(p)[0]}-processed.txt"
        print(f"Processing {os.path.basename(p)}")
        if os.path.exists(out_path):
            print("Already processed, skipping")

        cmd_line = [sys.executable, py_file, p]
        result = subprocess.run(cmd_line, capture_output=True, text=True)
        if not result.returncode:
            with open(out_path, "w", encoding="utf-8") as file:
                file.write(result.stdout)
        else:
            warnings.warn(f"Error processing {p}: {result.stderr}")

def load_results(
        path: str,
        ids: int | list[int] | None = None,
        driver: str | list[str] | None = None,
        protocol: str | list[str] | None = None,
        packet_size: int | list[int] | None = None,
        samples: int | list[int] | None = None
) -> dict[str, list[pd.DataFrame]]:
    print("Loading results")

    # Ensure types
    if ids is not None and not isinstance(ids, list):
        ids = [ids]
    if driver is not None and not isinstance(driver, list):
        driver = [driver]
    if protocol is not None and not isinstance(protocol, list):
        protocol = [protocol]
    if packet_size is not None and not isinstance(packet_size, list):
        packet_size = [packet_size]
    if samples is not None and not isinstance(samples, list):
        samples = [samples]

    # Load tests
    json_file = os.path.join(path, "tests.json")
    with open(json_file, "r") as f:
        tests = json.load(f)

    # Filter tests
    tests = filter_tests(
        tests,
        ids=ids,
        driver=driver,
        protocol=protocol,
        packet_size=packet_size,
        samples=samples
    )

    drivers = sorted(list(set([test["driver"] for test in tests])))
    results = {driver: [] for driver in drivers}
    for test in tests:
        results_file = f"{os.path.join(path, test["results"])}.csv"
        # noinspection argument-list
        df_benchmark = pd.read_csv(results_file)
        df_kernel = load_kernel_results(results_file)

        # noinspection bad-argument-type
        df = pd.concat([df_benchmark, df_kernel], axis=1)
        # noinspection unresolved-references
        df = df.reindex(df_benchmark.index)

        params = test.copy()
        for key in ["driver", "results"]:
            params.pop(key)
        for key, value in params.items():
            # noinspection unresolved-references
            df.attrs[key] = value

        results[test["driver"]].append(df)

    # Write results to output CSV
    with open(os.path.join(path, "results.txt"), "w") as f:
        num_drivers = len(results)
        for ii, (key, value) in enumerate(results.items(), start=1):
            for jj, df in enumerate(value, start=1):
                data_text = df.rename(columns=str.upper).to_string(index=False)
                text_width = len(data_text.split("\n")[0])
                divider_text = "-" * text_width

                f.write(f"TEST #{df.attrs['id']}\n")
                f.write(f"DRIVER: {key}, ")
                num_attrs = len(df.attrs.items())
                skip_attrs = ["id", "process"]
                for kk, (param_name, param_value) in enumerate(df.attrs.items(), start=1):
                    if param_name in skip_attrs:
                        continue
                    f.write(f"{param_name.upper()}: {param_value}")
                    if kk < num_attrs - len(skip_attrs):
                        f.write(", ")
                f.write("\n")
                f.write(f"{divider_text}\n")
                f.write(data_text)
                f.write("\n")
                if ii < num_drivers or jj < len(value):
                    f.write("\n")

    return results

def filter_tests(
        tests,
        ids: list[int] | None = None,
        driver: list[str] | None = None,
        protocol: list[str] | None = None,
        packet_size: list[int] | None = None,
        samples: list[int] | None = None
):
    tests_filtered = []
    for test in tests:
        if not test["process"]:
            continue
        if test["results"] is None:
            continue
        if ids is not None and test["id"] not in ids:
            continue
        if driver is not None and test["driver"] not in driver:
            continue
        if protocol is not None and test["protocol"] not in protocol:
            continue
        if packet_size is not None and test["packet_size"] not in packet_size:
            continue
        if samples is not None and test["samples"] not in samples:
            continue
        tests_filtered.append(test)
    return tests_filtered

def load_kernel_results(path):
    path = f"{os.path.splitext(path)[0]}-processed.txt"
    with open(path, "r", encoding="utf-8") as f:
        data = f.read()
        lines_grouped = data.split("\n\n")

        # Get the headers from the first group
        group = lines_grouped.pop(0)
        lines = group.split("\n")
        header_row = lines[0].replace(" ", "_")

        # Remove empty test results
        # We do this because we want the number of successfully completed tests
        # to equal the number of tests reported by the benchmarking script
        ii = 0
        while ii < len(lines_grouped):
            group = lines_grouped[ii]
            lines = group.split("\n")
            if len(lines) == 1:
                lines_grouped.pop(ii)
                ii -= 1
            ii += 1

        # Remove the last group
        lines_grouped.pop(len(lines_grouped) - 1)

        results_kernel = [header_row]
        for ii, group in enumerate(lines_grouped):
            lines = group.split("\n")

            data = ""
            for line in lines:
                if "ethernet_driver" in line:
                    data = line
                    break
            results_kernel.append(data)
        results_kernel = "\n".join(results_kernel)

        # noinspection argument-list
        df_kernel = pd.read_csv(StringIO(results_kernel))
        # noinspection unresolved-references
        df_kernel.drop(columns="Component", inplace=True)
    return df_kernel

def get_results_info(
        results
):
    drivers = sorted(list(results.keys()))

    protocols = set()
    for key, value in results.items():
        for test in value:
            protocols.add(test.attrs["protocol"])
    protocols = sorted(list(protocols))

    info = {
        "driver": drivers,
        "protocol": protocols
    }

    return info

def select_results(
        results,
        driver: str | list[str] | None = None,
        protocol: str | list[str] | None = None,
        packet_size: int | list[int] | None = None,
        samples: int | list[int] | None = None
):
    # Ensure types
    if driver is not None and not isinstance(driver, list):
        driver = [driver]
    if protocol is not None and not isinstance(protocol, list):
        protocol = [protocol]
    if packet_size is not None and not isinstance(packet_size, list):
        packet_size = [packet_size]
    if samples is not None and not isinstance(samples, list):
        samples = [samples]

    # Rebuild results structure adding only selected items
    results_selected = dict()
    for key, value in results.items():
        results_selected[key] = []
        for result in value:
            if (protocol is not None and
                result.attrs["protocol"] not in protocol):
                continue
            if (packet_size is not None and
                result.attrs["packet_size"] not in packet_size):
                continue
            if (samples is not None and
                result.attrs["samples"] not in samples):
                continue
            results_selected[key].append(result)
    if driver is not None:
        for key, value in results_selected.items():
            if key not in driver:
                results_selected.pop(key)
    return results_selected

def plot_results(
        results,
        out_folder,
        cols="protocol",
        x_var="Requested_Throughput"
):
    print("Plotting results")

    info = get_results_info(results)

    fig_perf = make_figure(
        results,
        info,
        [
            "Send_Throughput",
            "Receive_Throughput",
            "Bad_Packets",
            "RTT"
        ],
        cols,
        x_var
    )
    fig_perf.update_layout(
        height=plot_cfg["height"],
        width=plot_cfg["width"],
        title_text=f"Performance Results",
    )
    fig_perf.show()

    fig_perf.write_html(os.path.join(out_folder, "results_perf.html"))
    fig_perf.write_image(os.path.join(out_folder, "results_perf.png"))

    fig_util = make_figure(
        results,
        info,
        [
            "System_CPU_Util",
            "Kernel_CPU_Util",
            "User_CPU_Util",
            "Kernel_Entries"
        ],
        cols,
        x_var
    )
    fig_util.update_layout(
        height=plot_cfg["height"],
        width=plot_cfg["width"],
        title_text=f"Utilization Results",
    )
    fig_util.show()

    fig_util.write_html(os.path.join(out_folder, "results_util.html"))
    fig_util.write_image(os.path.join(out_folder, "results_util.png"))

def make_figure(
        results,
        info,
        metrics,
        cols,
        x_var
):
    num_rows = len(metrics)
    num_cols = len(info[cols])
    subplot_titles = (tuple([c.upper() for c in info[cols]]) +
                      ("",) * (num_rows * num_cols))

    fig = make_subplots(
        rows=num_rows,
        cols=num_cols,
        subplot_titles=subplot_titles,
    )
    for row, metric in enumerate(metrics, start=1):
        for col, name in enumerate(info[cols], start=1):
            kwargs = {
                cols: name
            }
            selected = select_results(results, **kwargs)

            show_legend = False
            if row == 1 and col == len(info[cols]):
                show_legend = True

            match metric:
                case "RTT":
                    plot_rtt(
                        selected,
                        fig,
                        row,
                        col,
                        x_var,
                        show_legend=show_legend
                    )
                case "System_CPU_Util":
                    plot_cpu_load(
                        selected,
                        fig,
                        row,
                        col,
                        x_var,
                        UtilizationType.SYSTEM,
                        show_legend=show_legend
                    )
                case "Kernel_CPU_Util":
                    plot_cpu_load(
                        selected,
                        fig,
                        row,
                        col,
                        x_var,
                        UtilizationType.KERNEL,
                        show_legend=show_legend
                    )
                case "User_CPU_Util":
                    plot_cpu_load(
                        selected,
                        fig,
                        row,
                        col,
                        x_var,
                        UtilizationType.USER,
                        show_legend=show_legend
                    )
                case _:
                    plot_generic(
                        selected,
                        fig,
                        row,
                        col,
                        x_var,
                        metric,
                        show_legend=show_legend
                    )

            if row == len(metrics):
                fig.update_xaxes(
                    title_text=x_var.upper(),
                    row=row,
                    col=col,
                )
            if col == 1:
                fig.update_yaxes(
                    title_text=metric.upper(),
                    row=row,
                    col=col,
                )
    return fig

def plot_rtt(
        results,
        fig,
        row,
        col,
        x_var,
        show_legend=False
):
    color_cntr = 0
    for driver, results_ in results.items():
        for df in results_:
            name = driver.title()
            if df.attrs["desc"] is not None:
                name = f"{name}: {df.attrs['desc'].title()}"

            # noinspection unresolved-references
            color_hex = plot_cfg["colors"][color_cntr]
            color_cntr += 1

            x = df[x_var]
            y = df["Median_RTT"]
            y_min = df["Minimum_RTT"]
            y_max = df["Maximum_RTT"]

            plot_add_min_max(
                fig,
                x,
                y_min,
                y_max,
                row,
                col,
                color_hex,
                name
            )
            plot_add_line(
                fig,
                x,
                y,
                row,
                col,
                color_hex,
                name,
                show_legend=show_legend
            )

def plot_cpu_load(
        results,
        fig,
        row,
        col,
        x_var,
        util_type,
        show_legend=False
):
    match util_type:
        case UtilizationType.SYSTEM:
            y_var = "Core_CPU_Util"
        case UtilizationType.KERNEL:
            y_var = "Core_Kernel_CPU_Util"
        case UtilizationType.USER:
            y_var = "Core_User_CPU_Util"
        case _:
            raise RuntimeError(f"Unknown UtilizationType: {util_type}")

    color_cntr = 0
    for driver, results_ in results.items():
        for df in results_:
            name = driver.title()
            if df.attrs["desc"] is not None:
                name = f"{name}: {df.attrs['desc'].title()}"

            # noinspection unresolved-references
            color_hex = plot_cfg["colors"][color_cntr]
            color_cntr += 1

            x = df[x_var]
            y = df[y_var]

            plot_add_line(
                fig,
                x,
                y,
                row,
                col,
                color_hex,
                name,
                show_legend=show_legend
            )

def plot_generic(
        results,
        fig,
        row,
        col,
        x_var,
        y_var,
        show_legend=False
):
    color_cntr = 0
    for driver, results_ in results.items():
        for df in results_:
            name = driver.title()
            if df.attrs["desc"] is not None:
                name = f"{name}: {df.attrs['desc'].title()}"

            # noinspection unresolved-references
            color_hex = plot_cfg["colors"][color_cntr]
            color_cntr += 1

            x = df[x_var]
            y = df[y_var]

            plot_add_line(
                fig,
                x,
                y,
                row,
                col,
                color_hex,
                name,
                show_legend=show_legend
            )

def plot_add_line(
        fig,
        x,
        y,
        row,
        col,
        color_hex,
        name,
        show_legend=False
):
    # noinspection bad-argument-type
    fig.add_trace(
        go.Scatter(
            x=x,
            y=y,
            mode="lines+markers",
            line_color=hex_to_rgb_str(color_hex),
            marker=dict(
                size=plot_cfg["marker_size"],
                color=hex_to_rgb_str(color_hex),
            ),
            showlegend=show_legend,
            name=name,
        ),
        row=row,
        col=col,
    )

def plot_add_min_max(
        fig,
        x,
        y_min,
        y_max,
        row,
        col,
        color_hex,
        name
):
    # noinspection bad-argument-type,SpellCheckingInspection
    fig.add_trace(
        go.Scatter(
            x=pd.concat([x, x[::-1]], ignore_index=True),
            y=pd.concat([y_max, y_min[::-1]], ignore_index=True),
            fill="toself",
            fillcolor=lighten_hex(color_hex),
            line_color="rgba(255,255,255,0)",
            showlegend=False,
            hoverinfo="skip",
            name=name,
        ),
        row=row,
        col=col,
    )

def hex_to_rgb(
        color_hex
):
    color_hex = color_hex.lstrip("#")
    return tuple(int(color_hex[i:i + 2], 16) for i in (0, 2, 4))

def hex_to_rgb_str(
        color_hex,
        alpha=1
):
    color_hex = color_hex.lstrip("#")
    r, g, b = hex_to_rgb(color_hex)
    return f"rgba({r},{g},{b},{alpha})"

def lighten_hex(
        color_hex,
        factor=0.1,
        alpha=0.5
):
    r, g, b = hex_to_rgb(color_hex)
    r = int(r + (255 - r) * factor)
    g = int(g + (255 - g) * factor)
    b = int(b + (255 - b) * factor)
    return f"rgba({r},{g},{b},{alpha})"

def run_proc_ipbench_results(path):
    process_serial_logs(path)

    results = load_results(path)
    plot_results(results, path)

# --- Main ---
if __name__ == "__main__":
    args_in = parse_args()

    run_proc_ipbench_results(
        args_in.input
    )
