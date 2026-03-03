window.BENCHMARK_DATA = {
  "lastUpdate": 1772532805811,
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
          "id": "2ef1d637316f38fbd191bde548a64bacfa6f61d3",
          "message": "Merge branch 'zcash:main' into dashified",
          "timestamp": "2026-03-03T17:01:09+07:00",
          "tree_id": "1744f7aee4af03e1aa95093e159d2e7dfbe6e334",
          "url": "https://github.com/dashpay/orchard/commit/2ef1d637316f38fbd191bde548a64bacfa6f61d3"
        },
        "date": 1772532805197,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2665031241,
            "range": "± 15612488",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2650125897,
            "range": "± 2594767",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 3793518140,
            "range": "± 9149713",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 4947763496,
            "range": "± 10935055",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 21208594,
            "range": "± 103404",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 21093750,
            "range": "± 112669",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 24300235,
            "range": "± 119722",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 27426892,
            "range": "± 128700",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1324994,
            "range": "± 15677",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 110307,
            "range": "± 256",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1322102,
            "range": "± 6988",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1161528856,
            "range": "± 1507529",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 14007304,
            "range": "± 26030",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 1903012,
            "range": "± 11164",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 13988094,
            "range": "± 21949",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 1858068,
            "range": "± 6155",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 69928181,
            "range": "± 191898",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 9387213,
            "range": "± 15234",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 69832092,
            "range": "± 105608",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 9232505,
            "range": "± 15146",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 139818154,
            "range": "± 131925",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 18693942,
            "range": "± 48850",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 139637308,
            "range": "± 1094895",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 18399464,
            "range": "± 41799",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 406365,
            "range": "± 2630",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 433318,
            "range": "± 534",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}