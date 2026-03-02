window.BENCHMARK_DATA = {
  "lastUpdate": 1772454217356,
  "repoUrl": "https://github.com/dashpay/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "quantum@dash.org",
            "name": "QuantumExplorer",
            "username": "QuantumExplorer"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c6d8faf20739e409511de434823c4590d6898ea4",
          "message": "feat: make memo size generic via MemoSize trait (#1)\n\n* Make memo size generic via MemoSize trait\n\nIntroduce a `MemoSize` trait that parameterizes memo-dependent sizes\nacross the Orchard protocol, allowing different protocols (e.g. Zcash\nwith 512-byte memos, Dash with 36-byte memos) to share the same\nimplementation. The memo is encrypted outside the ZK circuit via\nChaCha20-Poly1305 AEAD, so changing its size has zero impact on the\nHalo2 proof system.\n\nAll generic types use `M: MemoSize = ZcashMemo` default parameters\nfor full backward compatibility. PCZT types remain at ZcashMemo.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix\n\n* use non local note encryption crate\n\n* export pub use zcash_note_encryption\n\n* Make PCZT types generic over MemoSize\n\nPropagate the MemoSize type parameter through all PCZT types (Bundle,\nAction, Output) and their impl blocks across parse, tx_extractor,\nio_finalizer, verify, updater, signer, and prover modules. This allows\nPCZT to work with non-default memo sizes like DashMemo.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-02T19:10:52+07:00",
          "tree_id": "1beb5f85e5f4ebfee012ee52884a68f19fbca809",
          "url": "https://github.com/dashpay/orchard/commit/c6d8faf20739e409511de434823c4590d6898ea4"
        },
        "date": 1772454216259,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2699680943,
            "range": "± 88829413",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2720509451,
            "range": "± 25957149",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 3854796628,
            "range": "± 10057973",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5044969550,
            "range": "± 11721500",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 21026139,
            "range": "± 147221",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 21064906,
            "range": "± 173777",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 24269633,
            "range": "± 192128",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 27916441,
            "range": "± 253794",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1477027,
            "range": "± 10804",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 125711,
            "range": "± 141",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1475532,
            "range": "± 6013",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1343055480,
            "range": "± 1514080",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 15610992,
            "range": "± 39184",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2132106,
            "range": "± 6113",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 15582915,
            "range": "± 36676",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2096865,
            "range": "± 8068",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 78029915,
            "range": "± 137564",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10607150,
            "range": "± 19439",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 77856544,
            "range": "± 221726",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10427507,
            "range": "± 25219",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 155944098,
            "range": "± 943123",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21193031,
            "range": "± 178580",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 155683201,
            "range": "± 633310",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 20837219,
            "range": "± 49705",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 461969,
            "range": "± 3503",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 488197,
            "range": "± 1707",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}