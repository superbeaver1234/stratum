#!/usr/bin/env python3
"""Capture pinned DigiByte GBT and MultiShield historical fixtures.

Research utility only. It never submits work and never prints RPC credentials.

Environment:
  DGB_RPC_URL       e.g. http://127.0.0.1:14022
  DGB_RPC_USER
  DGB_RPC_PASSWORD

Example:
  DGB_RPC_URL=http://127.0.0.1:14022 \
  DGB_RPC_USER=fixture DGB_RPC_PASSWORD='...' \
  python3 tools/capture_dgb_consensus.py \
      --span 900 --output tests/fixtures/dgb/mainnet-v9.26.5.json
"""

from __future__ import annotations

import argparse
import base64
import json
import os
import sys
import urllib.error
import urllib.request
from dataclasses import dataclass
from pathlib import Path
from typing import Any

PINNED_VERSION = 9260500
PINNED_SUBVERSION_PREFIX = "/DigiByte Core:9.26.5/"
PINNED_COMMIT = "05b50e229db5a3d1fb316c77f3f6c62efa879b96"
SHA256D_VERSION_BITS = 0x0200
ALGO_MASK = 0x0F00
ALGO_BY_BITS = {
    0x0000: "scrypt",
    0x0200: "sha256d",
    0x0400: "groestl",
    0x0600: "skein",
    0x0800: "qubit",
    0x0E00: "odo",
}


class RpcFailure(RuntimeError):
    pass


@dataclass(frozen=True)
class RpcConfig:
    url: str
    user: str
    password: str


class RpcClient:
    def __init__(self, config: RpcConfig, timeout: float) -> None:
        self._config = config
        self._timeout = timeout
        token = base64.b64encode(f"{config.user}:{config.password}".encode()).decode()
        self._authorization = f"Basic {token}"
        self._id = 0

    def call(self, method: str, params: list[Any] | None = None) -> Any:
        self._id += 1
        payload = json.dumps(
            {"jsonrpc": "1.0", "id": self._id, "method": method, "params": params or []},
            separators=(",", ":"),
        ).encode()
        request = urllib.request.Request(
            self._config.url,
            data=payload,
            method="POST",
            headers={
                "Authorization": self._authorization,
                "Content-Type": "application/json",
                "User-Agent": "stratum-consensus-fixture/0.1",
            },
        )
        try:
            with urllib.request.urlopen(request, timeout=self._timeout) as response:
                if response.length is not None and response.length > 32 * 1024 * 1024:
                    raise RpcFailure(f"oversized response for {method}: {response.length} bytes")
                raw = response.read(32 * 1024 * 1024 + 1)
        except (urllib.error.URLError, TimeoutError) as exc:
            raise RpcFailure(f"transport failure calling {method}: {type(exc).__name__}") from exc
        if len(raw) > 32 * 1024 * 1024:
            raise RpcFailure(f"oversized response for {method}")
        decoded = json.loads(raw)
        if decoded.get("error") is not None:
            error = decoded["error"]
            code = error.get("code") if isinstance(error, dict) else None
            message = error.get("message") if isinstance(error, dict) else str(error)
            raise RpcFailure(f"RPC {method} failed: code={code} message={message}")
        return decoded["result"]


def require_env(name: str) -> str:
    value = os.environ.get(name)
    if not value:
        raise SystemExit(f"missing required environment variable {name}")
    return value


def algo_from_version(version: int) -> str:
    return ALGO_BY_BITS.get(version & ALGO_MASK, "unknown")


def capture(args: argparse.Namespace) -> dict[str, Any]:
    client = RpcClient(
        RpcConfig(
            url=require_env("DGB_RPC_URL"),
            user=require_env("DGB_RPC_USER"),
            password=require_env("DGB_RPC_PASSWORD"),
        ),
        timeout=args.timeout,
    )

    network = client.call("getnetworkinfo")
    chain = client.call("getblockchaininfo")
    if network.get("version") != PINNED_VERSION:
        raise RpcFailure(
            f"wrong DigiByte Core version: expected {PINNED_VERSION}, got {network.get('version')}"
        )
    if not str(network.get("subversion", "")).startswith(PINNED_SUBVERSION_PREFIX):
        raise RpcFailure(f"unexpected daemon subversion: {network.get('subversion')!r}")
    if chain.get("chain") != "main":
        raise RpcFailure(f"fixture capture requires mainnet, got {chain.get('chain')!r}")
    if chain.get("initialblockdownload"):
        raise RpcFailure("refusing fixture capture while daemon is in initial block download")

    gbt = client.call("getblocktemplate", [{"rules": ["segwit"]}, "sha256d"])
    if gbt.get("pow_algo_id") != 0 or gbt.get("pow_algo") != "sha256d":
        raise RpcFailure(
            "daemon did not return a SHA256d GBT: "
            f"pow_algo_id={gbt.get('pow_algo_id')!r} pow_algo={gbt.get('pow_algo')!r}"
        )

    best_height = int(client.call("getblockcount"))
    end_height = args.end_height if args.end_height is not None else best_height
    if end_height > best_height:
        raise RpcFailure(f"requested end height {end_height} exceeds tip {best_height}")
    if args.span < 200:
        raise RpcFailure("--span must be at least 200 blocks to provide MTP/algo warmup")
    start_height = end_height - args.span + 1
    if start_height < 1_430_000:
        raise RpcFailure("fixture range must stay entirely inside DigiByte V4 era")

    headers: list[dict[str, Any]] = []
    for height in range(start_height, end_height + 1):
        block_hash = client.call("getblockhash", [height])
        header = client.call("getblockheader", [block_hash, True])
        version = int(header["version"])
        headers.append(
            {
                "height": height,
                "hash": block_hash,
                "timestamp": int(header["time"]),
                "bits": str(header["bits"]).lower(),
                "version": version,
                "algo": algo_from_version(version),
            }
        )

    vectors: list[dict[str, Any]] = []
    # A real SHA256d block at height H proves the exact SHA256d nBits that Core
    # computed from tip H-1. Keep 128 blocks of warmup so each vector can replay
    # MTP and find a prior SHA block without relying on derived explorer values.
    warmup_height = start_height + 128
    for header in headers:
        if header["height"] <= warmup_height:
            continue
        if header["algo"] != "sha256d":
            continue
        vectors.append(
            {
                "tip_height": header["height"] - 1,
                "solved_height": header["height"],
                "expected_next_sha256d_bits": header["bits"],
            }
        )

    if len(vectors) < args.min_vectors:
        raise RpcFailure(
            f"range produced only {len(vectors)} SHA256d vectors; "
            f"need {args.min_vectors}. Increase --span."
        )

    return {
        "fixture_format": 1,
        "authority": {
            "repository": "DigiByte-Core/digibyte",
            "tag": "v9.26.5",
            "commit": PINNED_COMMIT,
        },
        "daemon": {
            "version": network["version"],
            "subversion": network["subversion"],
            "protocolversion": network.get("protocolversion"),
            "chain": chain["chain"],
            "best_height_at_capture": best_height,
            "best_block_hash": chain.get("bestblockhash"),
        },
        "sha256d_gbt": gbt,
        "history": {
            "start_height": start_height,
            "end_height": end_height,
            "headers": headers,
            "vectors": vectors,
        },
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--span", type=int, default=900)
    parser.add_argument("--min-vectors", type=int, default=100)
    parser.add_argument("--end-height", type=int)
    parser.add_argument("--timeout", type=float, default=10.0)
    args = parser.parse_args()

    try:
        fixture = capture(args)
    except RpcFailure as exc:
        print(f"capture failed: {exc}", file=sys.stderr)
        return 1

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(fixture, indent=2, sort_keys=True) + "\n")
    print(
        f"wrote {args.output}: "
        f"{len(fixture['history']['headers'])} headers, "
        f"{len(fixture['history']['vectors'])} SHA256d vectors"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
