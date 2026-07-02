window.BENCHMARK_DATA = {
  "lastUpdate": 1782974048142,
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
      },
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
          "id": "765f31a7c01c68a22f75fc6fec599ec292e00efb",
          "message": "docs: publish Dash Orchard book to GitHub Pages (#5)\n\nUpdate book.toml with Dash authorship and title. Modernize the book\ndeployment workflow to use the official GitHub Pages actions\n(upload-pages-artifact + deploy-pages) instead of the deprecated\npeaceiris/actions-gh-pages@v3. This requires enabling GitHub Pages\nwith \"GitHub Actions\" as the source in the repository settings.\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-05-24T22:59:04+07:00",
          "tree_id": "89a90956253d05a658e6eb01179241c5bd58bf37",
          "url": "https://github.com/dashpay/orchard/commit/765f31a7c01c68a22f75fc6fec599ec292e00efb"
        },
        "date": 1779639084177,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2766864885,
            "range": "± 86511068",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2766485233,
            "range": "± 13769860",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 3952766823,
            "range": "± 32979933",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5115719403,
            "range": "± 21040673",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 22312809,
            "range": "± 215585",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 22305236,
            "range": "± 155880",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 25956539,
            "range": "± 162575",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 29286444,
            "range": "± 258188",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1584191,
            "range": "± 8933",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 134160,
            "range": "± 278",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1581459,
            "range": "± 28765",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1412864071,
            "range": "± 2603793",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16743665,
            "range": "± 35953",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2286377,
            "range": "± 49779",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16714405,
            "range": "± 25985",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2244625,
            "range": "± 15002",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 83688715,
            "range": "± 99513",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 11368128,
            "range": "± 269245",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 83571715,
            "range": "± 1485715",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 11165544,
            "range": "± 24310",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 167380984,
            "range": "± 192233",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 22733073,
            "range": "± 40838",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 167077770,
            "range": "± 123399",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 22311939,
            "range": "± 24206",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 485206,
            "range": "± 4978",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 521674,
            "range": "± 714",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "quantum@dash.org",
            "name": "QuantumExplorer",
            "username": "QuantumExplorer"
          },
          "committer": {
            "email": "quantum@dash.org",
            "name": "Quantum Explorer",
            "username": "QuantumExplorer"
          },
          "distinct": true,
          "id": "f05557390a5843bc4eb04c66d8140bc9ef0fe9b7",
          "message": "docs: publish Dash Orchard book to GitHub Pages (#5)\n\nUpdate book.toml with Dash authorship and title. Modernize the book\ndeployment workflow to use the official GitHub Pages actions\n(upload-pages-artifact + deploy-pages) instead of the deprecated\npeaceiris/actions-gh-pages@v3. This requires enabling GitHub Pages\nwith \"GitHub Actions\" as the source in the repository settings.\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-06-03T19:34:28+02:00",
          "tree_id": "7ebb98a164e88e1a2e5c15aefbc209661eba2adf",
          "url": "https://github.com/dashpay/orchard/commit/f05557390a5843bc4eb04c66d8140bc9ef0fe9b7"
        },
        "date": 1780551814470,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2763503254,
            "range": "± 22559914",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2726520792,
            "range": "± 13784924",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 3893303692,
            "range": "± 20137922",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5103009375,
            "range": "± 45674194",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 22246131,
            "range": "± 157113",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 22328573,
            "range": "± 155888",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 25598328,
            "range": "± 216226",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 29115514,
            "range": "± 320691",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1582024,
            "range": "± 8787",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 134180,
            "range": "± 213",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1579284,
            "range": "± 7029",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1418940416,
            "range": "± 1476835",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16707843,
            "range": "± 40414",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2287438,
            "range": "± 4928",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16680151,
            "range": "± 78630",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2246686,
            "range": "± 7479",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 83511563,
            "range": "± 184235",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 11377640,
            "range": "± 16000",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 83358265,
            "range": "± 89185",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 11172640,
            "range": "± 167201",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 166916347,
            "range": "± 1140786",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 22746685,
            "range": "± 388279",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 166698913,
            "range": "± 318687",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 22335761,
            "range": "± 1221635",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 485267,
            "range": "± 930",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 521865,
            "range": "± 8273",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "81ca424e4d13dd1fa61fda24fdd1b3a03e4c01ab",
          "message": "fix: make PCZT trial decryption generic over MemoSize (#6)\n\nOrchardDomain::for_pczt_action and the ShieldedOutput impl for\npczt::Action were pinned to ZcashMemo, so a pczt::Bundle<DashMemo>\nbuilt via Builder::<DashMemo>::build_for_pczt had no supported\ntrial-decryption path: a Signer could not verify the outputs it was\nasked to authorize, and orphan rules prevent downstream crates from\nadding the impl themselves.\n\nGeneralize both over M: MemoSize, delegate for_pczt_action to\nfor_nullifier instead of duplicating its body, and add a regression\ntest that trial-decrypts a DashMemo PCZT output end-to-end.\n\nCo-authored-by: Claude Fable 5 <noreply@anthropic.com>",
          "timestamp": "2026-07-02T13:22:27+07:00",
          "tree_id": "99a3e8648a3a342124f1668f4cf4e864eba92531",
          "url": "https://github.com/dashpay/orchard/commit/81ca424e4d13dd1fa61fda24fdd1b3a03e4c01ab"
        },
        "date": 1782974046981,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2641668469,
            "range": "± 12755267",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2633334023,
            "range": "± 9742099",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 3789915446,
            "range": "± 12241299",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 4930922584,
            "range": "± 5588863",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 20766671,
            "range": "± 84518",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 20987551,
            "range": "± 212572",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 23972994,
            "range": "± 141405",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 26972603,
            "range": "± 129135",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1303426,
            "range": "± 127200",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 108556,
            "range": "± 147",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1300871,
            "range": "± 14235",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1149201379,
            "range": "± 2831621",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 13759644,
            "range": "± 18138",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 1870744,
            "range": "± 20076",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 13742248,
            "range": "± 20705",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 1827615,
            "range": "± 4671",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 68783538,
            "range": "± 84159",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 9197157,
            "range": "± 34578",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 68700206,
            "range": "± 126967",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 9051373,
            "range": "± 21542",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 137554855,
            "range": "± 588667",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 18345020,
            "range": "± 227298",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 137327929,
            "range": "± 210916",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 18053009,
            "range": "± 25417",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 396436,
            "range": "± 4427",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 429511,
            "range": "± 1080",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}