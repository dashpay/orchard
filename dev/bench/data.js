window.BENCHMARK_DATA = {
  "lastUpdate": 1779626545647,
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
          "id": "a7244b0e7894d3f1f2641a046a4023e2b1f3b2b8",
          "message": "docs: add CLAUDE.md for Claude Code guidance (#4)\n\nProvides build/test commands, feature flags, architecture overview,\nand key conventions to help Claude Code work effectively in this repo.\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-03T19:10:57+07:00",
          "tree_id": "ece166d8b4488aedbd56bd8ff140fcd52be5d50f",
          "url": "https://github.com/dashpay/orchard/commit/a7244b0e7894d3f1f2641a046a4023e2b1f3b2b8"
        },
        "date": 1772540620106,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2694160912,
            "range": "± 195264317",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2696879656,
            "range": "± 3729418",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 3859767560,
            "range": "± 9528007",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5031302929,
            "range": "± 18337613",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 20987975,
            "range": "± 146841",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 21035363,
            "range": "± 221915",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 24460858,
            "range": "± 188581",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 27679691,
            "range": "± 243373",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1486296,
            "range": "± 5676",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 125642,
            "range": "± 310",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1483166,
            "range": "± 9389",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1330782054,
            "range": "± 1045007",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 15701460,
            "range": "± 72445",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2132687,
            "range": "± 20959",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 15660401,
            "range": "± 47815",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2097248,
            "range": "± 8001",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 78440876,
            "range": "± 282654",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10611409,
            "range": "± 40206",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 78262363,
            "range": "± 461294",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10430776,
            "range": "± 31874",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 156863922,
            "range": "± 331129",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21208935,
            "range": "± 80462",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 156470567,
            "range": "± 312030",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 20851710,
            "range": "± 29056",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 461961,
            "range": "± 2926",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 488674,
            "range": "± 1230",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "quantum@dash.org",
            "name": "Quantum Explorer",
            "username": "QuantumExplorer"
          },
          "committer": {
            "email": "quantum@dash.org",
            "name": "Quantum Explorer",
            "username": "QuantumExplorer"
          },
          "distinct": true,
          "id": "09105216fd68d27afdff80b20d9543738fe0686b",
          "message": "ci: target dashified branch in benchmarks workflow\n\nThe rebase onto upstream 0.13.1 reverted the bench workflow trigger to\n`main`, which never runs on the dashpay fork. Retarget it to `dashified`.\n\nCo-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-05-24T19:25:53+07:00",
          "tree_id": "c6b3294dbbf110f03038d318e309a4604470a4fb",
          "url": "https://github.com/dashpay/orchard/commit/09105216fd68d27afdff80b20d9543738fe0686b"
        },
        "date": 1779626545178,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2769462706,
            "range": "± 57134899",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2768252271,
            "range": "± 8098240",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 3945112805,
            "range": "± 4591471",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5129886336,
            "range": "± 46907838",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 22380283,
            "range": "± 172501",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 22305903,
            "range": "± 175542",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 25741830,
            "range": "± 225798",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 29367055,
            "range": "± 248054",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1583426,
            "range": "± 3964",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 134193,
            "range": "± 324",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1584585,
            "range": "± 6874",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1422815091,
            "range": "± 801992",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16726397,
            "range": "± 17347",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2288634,
            "range": "± 3530",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16691807,
            "range": "± 22883",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2247203,
            "range": "± 2818",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 83577498,
            "range": "± 101785",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 11381191,
            "range": "± 17591",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 83401216,
            "range": "± 87849",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 11173608,
            "range": "± 18954",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 167135578,
            "range": "± 207613",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 22749942,
            "range": "± 32603",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 166794117,
            "range": "± 1149135",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 22332967,
            "range": "± 28160",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 485298,
            "range": "± 3207",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 521453,
            "range": "± 1003",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}