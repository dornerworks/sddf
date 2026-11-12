<!--
    Copyright 2026, DornerWorks

    SPDX-License-Identifier: BSD-2-Clause
-->

# Setup <!-- omit in toc -->

## Table of Contents <!-- omit in toc -->

- [1. Overview](#1-overview)
- [2. Preparing Images](#2-preparing-images)
- [3. Setting Up Benchmarking Environment](#3-setting-up-benchmarking-environment)
  - [3.1. Create `Dockerfile`](#31-create-dockerfile)
  - [3.2. Build Docker Image](#32-build-docker-image)
  - [3.3. Start `ipbench` Daemon](#33-start-ipbench-daemon)
  - [3.4. Run Benchmarking Tests](#34-run-benchmarking-tests)
    - [3.4.1. Run `echo_server` Example](#341-run-echo_server-example)
    - [3.4.2. Run `benchmark.py`](#342-run-benchmarkpy)
  - [3.5. Stop `ipbench` Daemon](#35-stop-ipbench-daemon)
- [4. Analyzing Results](#4-analyzing-results)

## 1. Overview

## 2. Preparing Images

Checkout branch `dev-tyler` from the [`dornerworks/sddf` repository](https://github.com/dornerworks/sddf).

```bash
git clone -b dev-tyler [<repository-url>](https://github.com/dornerworks/sddf)
```

Go to the `echo_server` example.

```bash
cd /path/to/sddf/examples/echo_server
```

Build the image. Use the `NETWORK_DRIVER` variable to select between the C, Rust, and Pancake drivers.

```bash
MICROKIT_BOARD=zcu102 MICROKIT_SDK=$HOME/tools/microkit-sdk-2.2.0 MICROKIT_CONFIG=benchmark NETWORK_DRIVER[=c|rust|pancake] make
```

## 3. Setting Up Benchmarking Environment

### 3.1. Create `Dockerfile`

```bash
vim Dockerfile
```

**Dockerfile**
```docker
FROM debian:trixie
RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential autoconf autoconf-archive automake libtool pkg-config \
        swig python3 python3-dev python3-lxml git ca-certificates \
        && rm -rf /var/lib/apt/lists/*
RUN git clone https://github.com/au-ts/ipbench.git /opt/ipbench \
        && cd /opt/ipbench/ipbench2 \
        && ./autogen.sh && ./configure && make -j && make install \
        && ldconfig
ENV PYTHONPATH=/usr/local/lib/python3.13/site-packages
WORKDIR /work
```

### 3.2. Build Docker Image

```bash
dzdo sudo docker build -t ipbench .
```

### 3.3. Start `ipbench` Daemon

```bash
dzdo sudo docker run -d --name ipbench --network host -v "$PWD":/work ipbench ipbenchd --debug
```

### 3.4. Run Benchmarking Tests

Repeat the following two steps for each test you wish to run. An example test matrix is below.

| Test No. | Driver | Protocol | Packet Size | Samples | Throughput |
| -------- | ------ | -------- | ----------- | ------- | ---------- |
| 1        | C      | UDP      | 1472        | 200000  | Default    |
| 2        | C      | TCP      | 1460        | 200000  | Default    |
| 3        | Rust   | UDP      | 1472        | 200000  | Default    |
| 4        | Rust   | TCP      | 1460        | 200000  | Default    |

#### 3.4.1. Run `echo_server` Example

Run the `echo_server` image on either the `zcu102` or `zynqmp` boards on the CI server. The processing scripts used in [4. Analyzing Results](#4-analyzing-results) require the serial logs, so start the hardware test using the `-l` option of the `hw-mgr` application.

```bash
hw-mgr test -u USERNAME -w [zcu102|zynqmp] -l LOG_FILE
```

Intercept the boot process and boot the image via `TFTP`.

```
dhcp; tftpboot 0x40000000 boot_files/board/loader.img; go 0x40000000
```

Wait for `client0` and `client1` to establish DHCP leases. Note the IP address of `client0` as it will be used in the next step.

#### 3.4.2. Run `benchmark.py`

Run the benchmarking script and wait for it to complete. This process may take anywhere from 10 to 30 minutes.

```bash
dzdo sudo docker exec -it -w /work ipbench python3 benchmark.py CLIENT0_IP_ADDR --clients localhost [--udp|--tcp]
```

### 3.5. Stop `ipbench` Daemon

```bash
dzdo sudo docker stop ipbench
dzdo sudo docker rm -f ipbench
```

## 4. Analyzing Results

The `analyze_results.py` script in the `scripts` directory generates performance and utilization plots comparing the benchmarking results of one or more drivers. It accepts a path to a directory as input where the directory contains the following contents.
- The `.csv` outputs from the `benchmark.py` script;
- The board serial logs with `.log` extensions; and
- A `test.json` file mapping each test to the corresponding test parameters.

The script requires the following packages.
- `pandas==3.0.6`
- `plotly==7.1.0`

An example input directory structure and `test.json` is shown below.

**`/path/to/results`**
```bash
.
├── output-2026-10-16T15-46-43.csv
├── output-2026-10-16T15-46-43.log
├── output-2026-10-16T16-03-30.csv
├── output-2026-10-16T16-03-30.log
├── output-2026-10-16T17-51-23.csv
├── output-2026-10-16T17-51-23.log
├── output-2026-10-16T18-18-30.csv
├── output-2026-10-16T18-18-30.log
└── tests.json
```

**`test.json`**
```json
[
    {
        "id": 1,
        "driver": "c",
        "protocol": "udp",
        "packet_size": 1472,
        "samples": 200000,
        "throughput_mbps": [10, 20, 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 970],
        "results":  "output-2026-10-16T15-46-43",
        "process": true
    },
    {
        "id": 2,
        "driver": "c",
        "protocol": "tcp",
        "packet_size": 1460,
        "samples": 200000,
        "throughput_mbps": [10, 20, 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 970],
        "results": "output-2026-10-16T16-03-30",
        "process": true
    },
    {
        "id": 3,
        "driver": "rust",
        "protocol": "udp",
        "packet_size": 1472,
        "samples": 200000,
        "throughput_mbps": [10, 20, 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 970],
        "results": "output-2026-10-16T17-51-23",
        "process": true
    },
    {
        "id": 4,
        "driver": "rust",
        "protocol": "tcp",
        "packet_size": 1460,
        "samples": 200000,
        "throughput_mbps": [10, 20, 50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 970],
        "results": "output-2026-10-16T18-18-30",
        "process": true
    }
]
```

To process these results, run

```bash
python proc_ipbench_results.py -i /path/to/results
```
