
## 2026-09-07T07:58:39+00:00 — official leaderboard read failed

- Trigger: before optimization experiments / resumed hypothesis plan. Source: https://api.ecdsa.fail/api/benchmarks/1ffb695a-309b-46b6-a728-2f97d8c7be74.
- Error: `URLError: <urlopen error [Errno -3] Temporary failure in name resolution>`. Prior public best None is stale; verified local best 1140989148. Current gap and rank unavailable.
- Continue local experiments; retry at the next scheduled checkpoint.

### 2026-09-07T07:58:42+00:00 — hypothesis `p000_public_control`

- Hypothesis: Reproduce public1196 with blocked fold disabled, untouched harness in its own workspace
- Result: **BUILD_OR_RUN_ERROR**, tested **None**; classical **None**, phase **None**, ancilla **None**.
- Avg T **None**, Q **None**; conditional product **None** (not a valid score unless CLEAN).
- Exact source/parameters, artifact and full unchanged benchmark log: `.autoresearch/hypothesis-loop/p000_public_control-1788767919977288007`.
- Source hashes unchanged: **True**. No automatic submission by this runner.

## 2026-09-07T07:59:51+00:00 — hypothesis-loop leaderboard check

- Trigger: before optimization experiments / resumed hypothesis plan.
- Public best: **1,139,101,740**, T **904049**, Q **1260**.
- Verified local best: **1,140,989,148**; gap **1,887,408** (**0.165693%**).
- Public target rank **#1**; local submission rank unavailable.
- Source ref: `1196b9fd0f2197c55411b22081b98d57759e2d66`.
- Official source: https://api.ecdsa.fail/api/benchmarks/1ffb695a-309b-46b6-a728-2f97d8c7be74

### 2026-09-07T08:00:04+00:00 — hypothesis `p000_public_control`

- Hypothesis: Reproduce public1196 with blocked fold disabled, untouched harness in its own workspace
- Result: **FAIL**, tested **9024**; classical **0**, phase **0**, ancilla **0**.
- Avg T **None**, Q **None**; conditional product **None** (not a valid score unless CLEAN).
- Exact source/parameters, artifact and full unchanged benchmark log: `.autoresearch/hypothesis-loop/p000_public_control-1788767991187596630`.
- Source hashes unchanged: **True**. No automatic submission by this runner.

## 2026-09-07T08:03:43+00:00 — hypothesis-loop leaderboard check

- Trigger: before optimization experiments / resumed hypothesis plan.
- Public best: **1,139,101,740**, T **904049**, Q **1260**.
- Verified local best: **1,140,989,148**; gap **1,887,408** (**0.165693%**).
- Public target rank **#1**; local submission rank unavailable.
- Source ref: `1196b9fd0f2197c55411b22081b98d57759e2d66`.
- Official source: https://api.ecdsa.fail/api/benchmarks/1ffb695a-309b-46b6-a728-2f97d8c7be74

### 2026-09-07T08:03:59+00:00 — hypothesis `p000_public_control`

- Hypothesis: Reproduce public1196 with blocked fold disabled, untouched harness in its own workspace
- Result: **CLEAN**, tested **9024**; classical **0**, phase **0**, ancilla **0**.
- Avg T **904048.96**, Q **1260**; conditional product **1139101740** (not a valid score unless CLEAN).
- Exact source/parameters, artifact and full unchanged benchmark log: `.autoresearch/hypothesis-loop/p000_public_control-1788768223787634837`.
- Source hashes unchanged: **True**. No automatic submission by this runner.

## 2026-09-07T08:04:00+00:00 — hypothesis-loop leaderboard check

- Trigger: three full evaluations or CLEAN result.
- Public best: **1,139,101,740**, T **904049**, Q **1260**.
- Verified local best: **1,139,101,740**; gap **0** (**0.000000%**).
- Public target rank **#1**; local submission rank unavailable.
- Source ref: `1196b9fd0f2197c55411b22081b98d57759e2d66`.
- Official source: https://api.ecdsa.fail/api/benchmarks/1ffb695a-309b-46b6-a728-2f97d8c7be74

## 2026-09-07T08:06:45+00:00 — hypothesis-loop leaderboard check

- Trigger: before optimization experiments / resumed hypothesis plan.
- Public best: **1,139,101,740**, T **904049**, Q **1260**.
- Verified local best: **1,139,101,740**; gap **0** (**0.000000%**).
- Public target rank **#1**; local submission rank unavailable.
- Source ref: `1196b9fd0f2197c55411b22081b98d57759e2d66`.
- Official source: https://api.ecdsa.fail/api/benchmarks/1ffb695a-309b-46b6-a728-2f97d8c7be74

### 2026-09-07T08:07:01+00:00 — hypothesis `p001_blocked_fold_robust_q1258`

- Hypothesis: Pareto space/time trade: exact block-boundary cleanup, wider value schedule and repair windows, longer convergence; fixed inherited nonce
- Result: **FAIL**, tested **9024**; classical **2**, phase **2**, ancilla **0**.
- Avg T **1067960.361**, Q **1296**; conditional product **1384076160** (not a valid score unless CLEAN).
- Exact source/parameters, artifact and full unchanged benchmark log: `.autoresearch/hypothesis-loop/p001_blocked_fold_robust_q1258-1788768405953655508`.
- Source hashes unchanged: **True**. No automatic submission by this runner.

## 2026-09-07T08:07:02+00:00 — hypothesis-loop leaderboard check

- Trigger: three full evaluations or CLEAN result.
- Public best: **1,139,101,740**, T **904049**, Q **1260**.
- Verified local best: **1,139,101,740**; gap **0** (**0.000000%**).
- Public target rank **#1**; local submission rank unavailable.
- Source ref: `1196b9fd0f2197c55411b22081b98d57759e2d66`.
- Official source: https://api.ecdsa.fail/api/benchmarks/1ffb695a-309b-46b6-a728-2f97d8c7be74

## 2026-09-07T08:13:20+00:00 — hypothesis-loop leaderboard check

- Trigger: before optimization experiments / resumed hypothesis plan.
- Public best: **1,139,101,740**, T **904049**, Q **1260**.
- Verified local best: **1,139,101,740**; gap **0** (**0.000000%**).
- Public target rank **#1**; local submission rank unavailable.
- Source ref: `1196b9fd0f2197c55411b22081b98d57759e2d66`.
- Official source: https://api.ecdsa.fail/api/benchmarks/1ffb695a-309b-46b6-a728-2f97d8c7be74

### 2026-09-07T08:13:37+00:00 — hypothesis `p002_blocked_endpoints_compact_compare`

- Hypothesis: Complete block carry architecture: endpoint correction plus exact 1-scratch phase comparator; widened walk and 720-round tail; fixed inherited nonce
- Result: **CLEAN**, tested **9024**; classical **0**, phase **0**, ancilla **0**.
- Avg T **1295073.699**, Q **1262**; conditional product **1634383388** (not a valid score unless CLEAN).
- Exact source/parameters, artifact and full unchanged benchmark log: `.autoresearch/hypothesis-loop/p002_blocked_endpoints_compact_compare-1788768800441917225`.
- Source hashes unchanged: **True**. No automatic submission by this runner.

## 2026-09-07T08:13:37+00:00 — hypothesis-loop leaderboard check

- Trigger: three full evaluations or CLEAN result.
- Public best: **1,139,101,740**, T **904049**, Q **1260**.
- Verified local best: **1,139,101,740**; gap **0** (**0.000000%**).
- Public target rank **#1**; local submission rank unavailable.
- Source ref: `1196b9fd0f2197c55411b22081b98d57759e2d66`.
- Official source: https://api.ecdsa.fail/api/benchmarks/1ffb695a-309b-46b6-a728-2f97d8c7be74

## 2026-09-07T08:21:28+00:00 — hypothesis-loop leaderboard check

- Trigger: before optimization experiments / resumed hypothesis plan.
- Public best: **1,139,101,740**, T **904049**, Q **1260**.
- Verified local best: **1,139,101,740**; gap **0** (**0.000000%**).
- Public target rank **#1**; local submission rank unavailable.
- Source ref: `1196b9fd0f2197c55411b22081b98d57759e2d66`.
- Official source: https://api.ecdsa.fail/api/benchmarks/1ffb695a-309b-46b6-a728-2f97d8c7be74

### 2026-09-07T08:21:31+00:00 — hypothesis `p003_streamed_blocks_budgeted_compare`

- Hypothesis: Stream correction operand bits directly through short carry blocks; use compact exact comparison only when its ladder would exceed cap; retain repaired 720-round schedule
- Result: **BUILD_OR_RUN_ERROR**, tested **None**; classical **None**, phase **None**, ancilla **None**.
- Avg T **None**, Q **None**; conditional product **None** (not a valid score unless CLEAN).
- Exact source/parameters, artifact and full unchanged benchmark log: `.autoresearch/hypothesis-loop/p003_streamed_blocks_budgeted_compare-1788769288581285112`.
- Source hashes unchanged: **True**. No automatic submission by this runner.

## 2026-09-07T08:24:48+00:00 — hypothesis-loop leaderboard check

- Trigger: before optimization experiments / resumed hypothesis plan.
- Public best: **1,139,101,740**, T **904049**, Q **1260**.
- Verified local best: **1,139,101,740**; gap **0** (**0.000000%**).
- Public target rank **#1**; local submission rank unavailable.
- Source ref: `1196b9fd0f2197c55411b22081b98d57759e2d66`.
- Official source: https://api.ecdsa.fail/api/benchmarks/1ffb695a-309b-46b6-a728-2f97d8c7be74

### 2026-09-07T08:25:06+00:00 — hypothesis `p004_low_space_replay_fallback`

- Hypothesis: Use exact MAJ/UMA full add only where the repaired replay cannot fit a carry ladder; preserves robust windows and introduces no short approximate chunk boundaries
- Result: **CLEAN**, tested **9024**; classical **0**, phase **0**, ancilla **0**.
- Avg T **1202279.432**, Q **1298**; conditional product **1560558142** (not a valid score unless CLEAN).
- Exact source/parameters, artifact and full unchanged benchmark log: `.autoresearch/hypothesis-loop/p004_low_space_replay_fallback-1788769488638476030`.
- Source hashes unchanged: **True**. No automatic submission by this runner.

## 2026-09-07T08:25:07+00:00 — hypothesis-loop leaderboard check

- Trigger: three full evaluations or CLEAN result.
- Public best: **1,139,101,740**, T **904049**, Q **1260**.
- Verified local best: **1,139,101,740**; gap **0** (**0.000000%**).
- Public target rank **#1**; local submission rank unavailable.
- Source ref: `1196b9fd0f2197c55411b22081b98d57759e2d66`.
- Official source: https://api.ecdsa.fail/api/benchmarks/1ffb695a-309b-46b6-a728-2f97d8c7be74

## 2026-09-07T08:26:54+00:00 — hypothesis-loop leaderboard check

- Trigger: before optimization experiments / resumed hypothesis plan.
- Public best: **1,139,101,740**, T **904049**, Q **1260**.
- Verified local best: **1,139,101,740**; gap **0** (**0.000000%**).
- Public target rank **#1**; local submission rank unavailable.
- Source ref: `1196b9fd0f2197c55411b22081b98d57759e2d66`.
- Official source: https://api.ecdsa.fail/api/benchmarks/1ffb695a-309b-46b6-a728-2f97d8c7be74

### 2026-09-07T08:27:11+00:00 — hypothesis `p005_whole_arithmetic_memory_cap`

- Hypothesis: Apply exact MAJ/UMA fallback to every ripple whose carry ladder exceeds the cap, including the non-splittable walk; same repaired schedule and windows
- Result: **CLEAN**, tested **9024**; classical **0**, phase **0**, ancilla **0**.
- Avg T **1203957.307**, Q **1255**; conditional product **1510966035** (not a valid score unless CLEAN).
- Exact source/parameters, artifact and full unchanged benchmark log: `.autoresearch/hypothesis-loop/p005_whole_arithmetic_memory_cap-1788769614078529951`.
- Source hashes unchanged: **True**. No automatic submission by this runner.

## 2026-09-07T08:27:12+00:00 — hypothesis-loop leaderboard check

- Trigger: three full evaluations or CLEAN result.
- Public best: **1,139,101,740**, T **904049**, Q **1260**.
- Verified local best: **1,139,101,740**; gap **0** (**0.000000%**).
- Public target rank **#1**; local submission rank unavailable.
- Source ref: `1196b9fd0f2197c55411b22081b98d57759e2d66`.
- Official source: https://api.ecdsa.fail/api/benchmarks/1ffb695a-309b-46b6-a728-2f97d8c7be74

## 2026-09-07T08:28:46+00:00 — hypothesis-loop leaderboard check

- Trigger: before optimization experiments / resumed hypothesis plan.
- Public best: **1,139,101,740**, T **904049**, Q **1260**.
- Verified local best: **1,139,101,740**; gap **0** (**0.000000%**).
- Public target rank **#1**; local submission rank unavailable.
- Source ref: `1196b9fd0f2197c55411b22081b98d57759e2d66`.
- Official source: https://api.ecdsa.fail/api/benchmarks/1ffb695a-309b-46b6-a728-2f97d8c7be74

### 2026-09-07T08:29:04+00:00 — hypothesis `p006_baked_envfree_publication`

- Hypothesis: Final env-free validation of baked Pareto defaults; require byte-identical P005 operation stream before publication
- Result: **CLEAN**, tested **9024**; classical **0**, phase **0**, ancilla **0**.
- Avg T **1203957.307**, Q **1255**; conditional product **1510966035** (not a valid score unless CLEAN).
- Exact source/parameters, artifact and full unchanged benchmark log: `.autoresearch/hypothesis-loop/p006_baked_envfree_publication-1788769726638365633`.
- Source hashes unchanged: **True**. No automatic submission by this runner.

## 2026-09-07T08:29:05+00:00 — hypothesis-loop leaderboard check

- Trigger: three full evaluations or CLEAN result.
- Public best: **1,139,101,740**, T **904049**, Q **1260**.
- Verified local best: **1,139,101,740**; gap **0** (**0.000000%**).
- Public target rank **#1**; local submission rank unavailable.
- Source ref: `1196b9fd0f2197c55411b22081b98d57759e2d66`.
- Official source: https://api.ecdsa.fail/api/benchmarks/1ffb695a-309b-46b6-a728-2f97d8c7be74
