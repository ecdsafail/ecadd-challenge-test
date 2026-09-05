"""Reproduce maximal CCX-run factoring from the exact public 9fd20de cache.

Requires Python 3.11+ and zstandard==0.23.0 (C backend). This is a cache-to-cache
reproducer, not a fresh generator run or a whole-circuit validity certificate.
All nine primitive kinds are retained; only adjacent ordinary CCX runs change.
The two original aggregate files are ignored because their references are stale.

Example:
  python recombine_maximal_runs.py --original-cache /path/to/original/cache \
      --output /path/to/new/output --expected-cache /path/to/candidate/cache
Use --index 0 for a bounded first-shard check; omit it for all 36 shards.
Inputs are authenticated against the public commit's actual Git blob IDs.
All output paths must be new. No network, compiler, native simulator or nonce.
"""
import argparse
from collections import Counter
import hashlib
import json
import os
from pathlib import Path
import resource
import signal
import stat
import struct
import sys
from types import SimpleNamespace

COMMIT = '9fd20de4eaa745859de468174dd6bd1f78b4bfb3'
GEN_SHA256 = '81c97fa2b978b690dcaab3f9c4360fbac714b9f5d2cf3c1e2cd4cead23b17601'
NAMES = {1:'x',2:'cx',3:'ccx',4:'z',5:'cz',6:'swap',7:'clean_c3x_mbu',
         8:'paired_tsub_compute_v1',9:'paired_tsub_uncompute_v1'}
ARITY = {1:1,2:2,3:3,4:1,5:2,6:2,7:5,8:3,9:3}
WORD, WITNESS = struct.Struct('<Q'), struct.Struct('<8Q')
MAX_FRAME = 128*1024**2
BLOBS = {
  "chunk-0001-0045.zst": {
    "git_blob_sha": "1060883099647caeca19b04de439b1af879beef3",
    "bytes": 44083
  },
  "chunk-0001-0045.zst.json": {
    "git_blob_sha": "a59a186e8106a3cedceb41947fc4f81e96c72e91",
    "bytes": 31086
  },
  "chunk-0046-0090.zst": {
    "git_blob_sha": "e1a9e48caacf25c331da042094e39ca65768ba0c",
    "bytes": 47802
  },
  "chunk-0046-0090.zst.json": {
    "git_blob_sha": "e8c4b4f80687acf9e8f9736e50b67eca27d61018",
    "bytes": 31932
  },
  "chunk-0091-0135.zst": {
    "git_blob_sha": "42278249efc97f0ceb52b252534c5e7356cb7d8e",
    "bytes": 50872
  },
  "chunk-0091-0135.zst.json": {
    "git_blob_sha": "d4c32f571224e319f4ba55a83d538aad76fd8cff",
    "bytes": 31958
  },
  "chunk-0136-0180.zst": {
    "git_blob_sha": "cb868132566faffc2d61a94394bee22f9e67e8c2",
    "bytes": 59834
  },
  "chunk-0136-0180.zst.json": {
    "git_blob_sha": "8344e935be0fe4e77be075fa0d1805be02f3066e",
    "bytes": 32018
  },
  "chunk-0181-0225.zst": {
    "git_blob_sha": "635b463d4c35bbfc3bc47b935d61e62267adb904",
    "bytes": 57471
  },
  "chunk-0181-0225.zst.json": {
    "git_blob_sha": "a41e9a5421075f11c15760253792fc35af2807af",
    "bytes": 32083
  },
  "chunk-0226-0270.zst": {
    "git_blob_sha": "c602f6164893a1f886ca2be77bfbf1aae75b0eeb",
    "bytes": 68687
  },
  "chunk-0226-0270.zst.json": {
    "git_blob_sha": "3db1bdc9f45247404a79a444f921527fb3e6691e",
    "bytes": 32141
  },
  "chunk-0271-0315.zst": {
    "git_blob_sha": "3aba22b77310cb3aacbbc323cf0d066b1ac2d8da",
    "bytes": 71778
  },
  "chunk-0271-0315.zst.json": {
    "git_blob_sha": "83c6dc78301ff8edcfb32ca32a8114dc3c3a22df",
    "bytes": 32140
  },
  "chunk-0316-0360.zst": {
    "git_blob_sha": "336e394f3bacd991cc9ca6e25cfa3cd8bc6159a7",
    "bytes": 81057
  },
  "chunk-0316-0360.zst.json": {
    "git_blob_sha": "0de72845e5692f4b45e714b32633e93ce8e4b5ed",
    "bytes": 32140
  },
  "chunk-0361-0405.zst": {
    "git_blob_sha": "045df4a464b61dc2306269dab8f6df6b96e1302c",
    "bytes": 75417
  },
  "chunk-0361-0405.zst.json": {
    "git_blob_sha": "aae9684e435bc9b2b97959c415e5af5a0b641e0f",
    "bytes": 32140
  },
  "chunk-0406-0450.zst": {
    "git_blob_sha": "01669ab83fed0f639cd4350c3839621cf2f456a0",
    "bytes": 67791
  },
  "chunk-0406-0450.zst.json": {
    "git_blob_sha": "8dc9021194ae81d132fd4817be8477e483ca7faa",
    "bytes": 32141
  },
  "chunk-0451-0495.zst": {
    "git_blob_sha": "124876a050a18c0dc6659057db6c878db6b74bdb",
    "bytes": 75033
  },
  "chunk-0451-0495.zst.json": {
    "git_blob_sha": "576691c2c225a16830784905324cb5a084f411e0",
    "bytes": 32142
  },
  "chunk-0496-0540.zst": {
    "git_blob_sha": "bbe599c8c2e8c8b311ea7653614501f3b4f2eb61",
    "bytes": 88657
  },
  "chunk-0496-0540.zst.json": {
    "git_blob_sha": "580c460c42d57f5fbde50b7168169c3d91ba280d",
    "bytes": 32142
  },
  "chunk-0541-0585.zst": {
    "git_blob_sha": "b931638c2be6d9a5b905cd149ad1e2710b78f956",
    "bytes": 77625
  },
  "chunk-0541-0585.zst.json": {
    "git_blob_sha": "66ed864e77482958b74c02e45ebbcce8e189ed33",
    "bytes": 32130
  },
  "chunk-0586-0630.zst": {
    "git_blob_sha": "db4e23c40491d6ffb15880b198112f99e40bcdcf",
    "bytes": 83469
  },
  "chunk-0586-0630.zst.json": {
    "git_blob_sha": "a656c42338d66fbc7728db75ac30aae21eda0852",
    "bytes": 32142
  },
  "chunk-0631-0675.zst": {
    "git_blob_sha": "5060e48c807f1f3cdb445644c673d3e13f25ef97",
    "bytes": 78939
  },
  "chunk-0631-0675.zst.json": {
    "git_blob_sha": "783b278b96b8d64b3261a75f0cc4be1fe8ccedc4",
    "bytes": 32143
  },
  "chunk-0676-0720.zst": {
    "git_blob_sha": "686d9a777678c12b37d040f15be5da63323e464c",
    "bytes": 86029
  },
  "chunk-0676-0720.zst.json": {
    "git_blob_sha": "8e2139d5870ec454c55dc6ed9e6982d77d0e18d6",
    "bytes": 32143
  },
  "chunk-0721-0765.zst": {
    "git_blob_sha": "3167481bf02bc069512fc18303cd8e6ff7ce0480",
    "bytes": 106852
  },
  "chunk-0721-0765.zst.json": {
    "git_blob_sha": "e092a58740cdedf49ea904a3072e3ec2e9c0d96d",
    "bytes": 32143
  },
  "chunk-0766-0810.zst": {
    "git_blob_sha": "3794864911b4390d0c1c3b5ec959f651fa2243f3",
    "bytes": 121711
  },
  "chunk-0766-0810.zst.json": {
    "git_blob_sha": "288561499d9b3e6d07743c848be906dab7fc6cf3",
    "bytes": 32143
  },
  "chunk-0811-0855.zst": {
    "git_blob_sha": "9f0d8e8cdf45ddaf27cc345c5d2dd62fc1f1728f",
    "bytes": 117885
  },
  "chunk-0811-0855.zst.json": {
    "git_blob_sha": "28bfbb590d5e17ac8fe3e33845abee36f0c9d08f",
    "bytes": 32143
  },
  "chunk-0856-0900.zst": {
    "git_blob_sha": "0415c90bb05100d45a31cff7db1f91988b67f761",
    "bytes": 103817
  },
  "chunk-0856-0900.zst.json": {
    "git_blob_sha": "38745e43b992a54b880059b1003c58ddb733fcb4",
    "bytes": 32144
  },
  "chunk-0901-0945.zst": {
    "git_blob_sha": "2b24407084750967bad2dfbde3587050ab175fb8",
    "bytes": 88957
  },
  "chunk-0901-0945.zst.json": {
    "git_blob_sha": "5c23cd4dfe769de89c7ba2a82a1111daa040af70",
    "bytes": 32142
  },
  "chunk-0946-0990.zst": {
    "git_blob_sha": "5b47e9ec310c2a2e4a2182ffc1cca891d6e619a4",
    "bytes": 87623
  },
  "chunk-0946-0990.zst.json": {
    "git_blob_sha": "44520aa72cc05fa38041a754ad4b867717328208",
    "bytes": 32130
  },
  "chunk-0991-1035.zst": {
    "git_blob_sha": "0b7e28ad0741352bf3f771878e8dca71b4455deb",
    "bytes": 85773
  },
  "chunk-0991-1035.zst.json": {
    "git_blob_sha": "60f46599b6fc36ef5c918eb55c23ede2c17ae3ca",
    "bytes": 32179
  },
  "chunk-1036-1080.zst": {
    "git_blob_sha": "5a38aac391ec363ca26dc47818dc2f435c980aac",
    "bytes": 122544
  },
  "chunk-1036-1080.zst.json": {
    "git_blob_sha": "7c3dbcb861d94aa1fb4891c9222f15df6185380f",
    "bytes": 32190
  },
  "chunk-1081-1125.zst": {
    "git_blob_sha": "76b3d071f83e526f679441d288d8f6b5c882f094",
    "bytes": 109044
  },
  "chunk-1081-1125.zst.json": {
    "git_blob_sha": "58a6c24a351c501dd01cb10121455398d2716425",
    "bytes": 32191
  },
  "chunk-1126-1170.zst": {
    "git_blob_sha": "35164fb0ad4663c5d2ec058456dedbe0129395f2",
    "bytes": 112619
  },
  "chunk-1126-1170.zst.json": {
    "git_blob_sha": "c32d69e033f25b419018c0d9ce914f7320c9c136",
    "bytes": 32190
  },
  "chunk-1171-1215.zst": {
    "git_blob_sha": "de9413f452d580ca531a261666041af80a2282c3",
    "bytes": 95952
  },
  "chunk-1171-1215.zst.json": {
    "git_blob_sha": "8b8f5822d6812ebdcdb9c8793a2c0797426cf931",
    "bytes": 32189
  },
  "chunk-1216-1260.zst": {
    "git_blob_sha": "27129f5b3031c41ca76cab8389a934f68ba5001e",
    "bytes": 113469
  },
  "chunk-1216-1260.zst.json": {
    "git_blob_sha": "535d0d250c554bdcecfa3427fb2f3a8189c20574",
    "bytes": 32190
  },
  "chunk-1261-1305.zst": {
    "git_blob_sha": "47395075dd1a430c6e1a0920ca603fcffb71c492",
    "bytes": 101665
  },
  "chunk-1261-1305.zst.json": {
    "git_blob_sha": "29494d57c4a4e470dffee315db06eb107d4b3b6e",
    "bytes": 32191
  },
  "chunk-1306-1350.zst": {
    "git_blob_sha": "c22c2edf0de4ef63b0a476c0c465e69084c681cf",
    "bytes": 99604
  },
  "chunk-1306-1350.zst.json": {
    "git_blob_sha": "b9ccfbbb0d1143951dd04ed64e0bf06078de220e",
    "bytes": 32190
  },
  "chunk-1351-1395.zst": {
    "git_blob_sha": "8494df4c3a810f94047a4f83295588331846906e",
    "bytes": 104130
  },
  "chunk-1351-1395.zst.json": {
    "git_blob_sha": "025d8bacf0e375ff72b3ef9f02c5bfc1cab6eec9",
    "bytes": 32174
  },
  "chunk-1396-1440.zst": {
    "git_blob_sha": "e38947fea8104515769404b5ad87cfb2a1d15dd7",
    "bytes": 86411
  },
  "chunk-1396-1440.zst.json": {
    "git_blob_sha": "9d0cefa3d2b62ba841f614e79c63322b6a34d436",
    "bytes": 32187
  },
  "chunk-1441-1485.zst": {
    "git_blob_sha": "a9b60c41014c5d43d997fcbc78943127686eacb4",
    "bytes": 81944
  },
  "chunk-1441-1485.zst.json": {
    "git_blob_sha": "bc608cde08a21d9f6c1e7e8b6dc59f2ced8316f5",
    "bytes": 32187
  },
  "chunk-1486-1530.zst": {
    "git_blob_sha": "94f1375f85306d87b6acf9bbf10165a1ec7e0682",
    "bytes": 81591
  },
  "chunk-1486-1530.zst.json": {
    "git_blob_sha": "c0552a0414709f21514bcca3f912dd0ceba7599b",
    "bytes": 32187
  },
  "chunk-1531-1575.zst": {
    "git_blob_sha": "5abdfb6c51b44e9b1a17e5db237594b0ca961453",
    "bytes": 75667
  },
  "chunk-1531-1575.zst.json": {
    "git_blob_sha": "dfba61066572892d6e0d0e721e8c4e5b93f1ff26",
    "bytes": 32187
  },
  "chunk-1576-1616.zst": {
    "git_blob_sha": "3929548b173ce3d05439eec2bddebeb41ddd38fb",
    "bytes": 81526
  },
  "chunk-1576-1616.zst.json": {
    "git_blob_sha": "c425096c816563d3e28eb9931ef5908ef2c945ab",
    "bytes": 29422
  }
}
def need(ok, message):
    if not ok:
        raise ValueError(message)
def sha(raw):
    return hashlib.sha256(raw).hexdigest()
def encoded(value):
    return (json.dumps(value,sort_keys=True,indent=2)+'\n').encode()
def read(path, limit=MAX_FRAME):
    path=Path(path);info=path.lstat()
    need(stat.S_ISREG(info.st_mode) and path.resolve()==path and info.st_size<=limit,
         'bounded regular canonical input')
    return path.read_bytes()
def authenticated(folder, name):
    row=BLOBS[name];raw=read(folder/name)
    need(len(raw)==row['bytes'] and hashlib.sha1(
        ('blob '+str(len(raw))+'\0').encode()+raw).hexdigest()==row['git_blob_sha'],
        'exact original Git blob '+name)
    return raw
def save(path,raw):
    with path.open('xb') as stream:
        stream.write(raw);stream.flush();os.fsync(stream.fileno())
def normalized(c):
    need(type(c)is dict and set(c)<=set(NAMES.values())
         and all(type(v)is int and v>=0 for v in c.values()),'nine count names')
    return {name:c.get(name,0) for name in NAMES.values()}
def summed(rows):
    total=dict.fromkeys(NAMES.values(),0)
    for row in rows:
        for name,value in normalized(row['counts']).items():
            total[name]+=value
    return total
def directional(c):
    c=normalized(c);base=c['ccx']+2*c['clean_c3x_mbu'];n=sum(c.values())
    return {d:dict(ccx=base+c[compute],hmr=c['clean_c3x_mbu']+c[erase],
        cz=c['clean_c3x_mbu']+c[erase],ops=n+3*c['clean_c3x_mbu']+c[erase])
        for d,compute,erase in (('forward',NAMES[8],NAMES[9]),('reverse',NAMES[9],NAMES[8]))}

# These five functions retain the reviewed maximal-run selection, tie order,
# parity order and witness layout. b/kernels are local compatibility namespaces,
# not external imports or executables.

def replacement(kind, fixed, items):
    odd = [q for q, count in Counter(items).items() if count % 2]
    if not odd:
        return []
    pivot = odd[0]
    if kind == 'control_xor':
        common, target = fixed
        ladder = [(q, pivot) for q in odd[1:]]
        middle = (common, pivot, target)
    else:
        ladder = [(pivot, q) for q in odd[1:]]
        middle = (*fixed, pivot)
    return ladder + [middle] + list(reversed(ladder))

def runs(ops):
    witnesses = []
    i = 0
    while i < len(ops):
        first = ops[i]
        if len(first) != 3:
            i += 1
            continue
        candidates = []
        for kind, fixed in [('control_xor', (first[0], first[2])), ('control_xor', (first[1], first[2])), ('target_fanout', tuple(sorted(first[:2])))]:
            j = i
            items = []
            while j < len(ops):
                q = ops[j]
                if len(q) != 3:
                    break
                if kind == 'control_xor':
                    common, target = fixed
                    if q[2] != target or common not in q[:2]:
                        break
                    item = q[1] if q[0] == common else q[0]
                else:
                    if tuple(sorted(q[:2])) != fixed:
                        break
                    item = q[2]
                assert item not in fixed
                items.append(item)
                j += 1
            if len(items) > 1:
                after = replacement(kind, fixed, items)
                saved = len(items) - sum((len(q) == 3 for q in after))
                candidates.append((saved, j - i, kind, fixed, items, after))
        if not candidates:
            i += 1
            continue
        saved, length, kind, fixed, items, after = max(candidates, key=lambda v: (v[0], v[1], -len(v[-1])))
        witnesses.append(dict(first=i, end=i + length, kind=kind, fixed=fixed, items=items, ccx_saved=saved, added_cx=sum((len(q) == 2 for q in after))))
        i += length
    return witnesses

def unpack(raw):
    b.need(len(raw) == 8, 'truncated record')
    word, = WORD.unpack(raw)
    kind = word & 15
    b.need(kind in NAMES and word >> 4 & 15 == ARITY[kind] and (word >> 8 + 10 * ARITY[kind] == 0), 'unknown or malformed typed primitive')
    operands = tuple((word >> 8 + 10 * i & 1023 for i in range(ARITY[kind])))
    b.need(len(set(operands)) == ARITY[kind] and max(operands) < 577, 'aliased/out-of-range operand')
    return (kind, operands)

def pack(kind, operands):
    b.need(kind in ARITY and len(operands) == ARITY[kind] and all((type(q) is int and 0 <= q < 577 for q in operands)) and (len(set(operands)) == len(operands)), 'typed pack operands')
    return WORD.pack(kind | ARITY[kind] << 4 | sum((q << 8 + 10 * i for i, q in enumerate(operands))))

def transform_step(raw, step, witness_start=0):
    """Bounded glue around the exact root kernels; ORIGINAL records only."""
    b.need(type(step) is int and 1 <= step <= 1616 and (type(witness_start) is int) and (witness_start >= 0), 'step/witness scope')
    b.need(raw and len(raw) % 8 == 0 and (len(raw) <= b.MAX_STEP_BYTES), 'step byte bound')
    typed = [unpack(raw[i:i + 8]) for i in range(0, len(raw), 8)]
    ops = [q if kind == 3 else () for kind, q in typed]
    kernel = kernels.transform_kernel()
    plans = kernel.runs(ops)
    output, witnesses = (bytearray(), bytearray())
    rules, lengths = (Counter(), Counter())
    cursor = removed = inserted = saved = added = duplicates = empty = 0
    for plan in plans:
        first, end = (plan['first'], plan['end'])
        b.need(cursor <= first < end <= len(ops) and end - first >= 2, 'original run partition')
        output.extend(raw[8 * cursor:8 * first])
        start_out = len(output) // 8
        replacement = kernel.replacement(plan['kind'], plan['fixed'], plan['items'])
        for operands in replacement:
            record = pack(len(operands), operands)
            unpack(record)
            output.extend(record)
        rule = {'control_xor': 1, 'target_fanout': 2}[plan['kind']]
        end_out = len(output) // 8
        ccx_saved = end - first - sum((len(q) == 3 for q in replacement))
        cx_added = sum((len(q) == 2 for q in replacement))
        b.need(ccx_saved == plan['ccx_saved'] and cx_added == plan['added_cx'], 'kernel delta')
        witnesses.extend(WITNESS.pack(first, end, start_out, end_out, rule, *plan['fixed'], ccx_saved))
        rules[str(rule)] += 1
        lengths[str(end - first)] += 1
        removed += end - first
        inserted += len(replacement)
        saved += ccx_saved
        added += cx_added
        duplicates += len(set(plan['items'])) != len(plan['items'])
        empty += not replacement
        cursor = end
    output.extend(raw[8 * cursor:])
    before_hist = Counter((kind for kind, _ in typed))
    before = {name: before_hist[kind] for kind, name in NAMES.items()}
    counts = Counter((NAMES[unpack(output[i:i + 8])[0]] for i in range(0, len(output), 8)))
    after = {k: counts[k] for k in NAMES.values()}
    b.need(after == dict(before, cx=before['cx'] + added, ccx=before['ccx'] - saved) and len(output) // 8 == len(ops) - removed + inserted, 'generic parity count identity')
    b.need(len(witnesses) == 64 * len(plans) and len(witnesses) <= b.MAX_WITNESS_BYTES and (len(output) <= 2 * len(raw)) and (len(output) <= b.MAX_STEP_BYTES), 'step output bounds')
    row = dict(step=step, counts=after, records=len(output) // 8, lowered_base_ccx=after['ccx'] + 2 * after['clean_c3x_mbu'], baseline_counts=before, baseline_records=len(ops), baseline_raw_record_sha256=hashlib.sha256(raw).hexdigest(), raw_record_sha256=hashlib.sha256(output).hexdigest(), selected_runs=len(plans), raw_ccx_saved=saved, added_cx=added, removed_records=removed, inserted_records=inserted, rule_counts={k: rules[k] for k in ('1', '2')}, run_lengths=dict(sorted(lengths.items(), key=lambda p: int(p[0]))), max_run_length=max((int(n) for n in lengths), default=0), duplicate_groups=duplicates, empty_parity_groups=empty, witness_record_start=witness_start, witness_record_end=witness_start + len(plans), witness_sha256=hashlib.sha256(witnesses).hexdigest())
    return (bytes(output), bytes(witnesses), row)

b=SimpleNamespace(need=need,MAX_STEP_BYTES=4*1024**2,MAX_WITNESS_BYTES=8*1024**2)
kernels=SimpleNamespace(transform_kernel=lambda:SimpleNamespace(runs=runs,replacement=replacement))

def counts(row):
    row = dict(row)
    c = row['counts']
    base = c['ccx'] + 2 * c['clean_c3x_mbu']
    need(row['lowered_base_ccx'] == base, 'ordinary and kind7 count')
    row.update(lowered_forward_ccx=base + c[NAMES[8]], lowered_reverse_ccx=base + c[NAMES[9]], lowered_forward_hmr=c['clean_c3x_mbu'] + c[NAMES[9]], lowered_reverse_hmr=c['clean_c3x_mbu'] + c[NAMES[8]], lowered_forward_ops=row['records'] + 3 * c['clean_c3x_mbu'] + c[NAMES[9]], lowered_reverse_ops=row['records'] + 3 * c['clean_c3x_mbu'] + c[NAMES[8]])
    return row

def decode_frame(zstd,raw,expected,unknown=False):
    need(24<=expected<=MAX_FRAME,'frame bound')
    p=zstd.get_frame_parameters(raw)
    need(p.dict_id==0 and 0<p.window_size<=MAX_FRAME and
         (p.content_size==expected or (unknown and p.content_size==zstd.CONTENTSIZE_UNKNOWN)),
         'single zstd frame parameters')
    body=zstd.ZstdDecompressor(max_window_size=MAX_FRAME).decompress(
        raw,max_output_size=expected,read_across_frames=False,allow_extra_data=False)
    need(len(body)==expected,'exact frame length')
    return body

def reproduce(index, original, zstd):
    need(type(index)is int and 0<=index<36,'shard index')
    start=45*index+1;end=min(start+44,1616);name=f'chunk-{start:04d}-{end:04d}.zst'
    old=authenticated(original,name);meta_raw=authenticated(original,name+'.json');meta=json.loads(meta_raw)
    need(meta['schema']=='paper2607-paired-directional-stream-v1' and
        (meta['n'],meta['qubits'],meta['aux_size'],meta['record_bytes'],meta['schedule_end'])==(256,577,11,8,1616)
        and meta['measurement_uncompute'] is False,'original nine-kind metadata ABI')
    need((meta['step_start'],meta['step_end'])==(start,end) and
         meta['compressed_bytes']==len(old) and meta['compressed_sha256']==sha(old),'source sidecar join')
    need([r['step'] for r in meta['per_step']]==list(range(start,end+1)),'exact source step partition')
    need(sum(r['records'] for r in meta['per_step'])==meta['records'] and
         summed(meta['per_step'])==normalized(meta['counts']) and
         directional(meta['counts'])==meta['directional'],'original shard counts')
    body=decode_frame(zstd,old,24+8*meta['records'],unknown=True)
    header=b'P26EEA3\0'+struct.pack('<4I',256,577,start,end)
    need(body[:24]==header and sha(body[24:])==meta['raw_record_sha256'],'original header and raw body')
    candidate=bytearray(header);witness=bytearray();rows=[];cursor=24
    for before in meta['per_step']:
        size=8*before['records']
        need(0<size<=b.MAX_STEP_BYTES,'step size bound')
        raw=body[cursor:cursor+size];cursor+=size
        need(len(raw)==size and sha(raw)==before['raw_record_sha256'],'exact original step bytes')
        out,w,row=transform_step(raw,before['step']);row=counts(row)
        need(row['baseline_counts']==normalized(before['counts']) and
             directional(before['counts'])==before['directional'],'original step count join')
        candidate.extend(out);witness.extend(w);rows.append(row)
    need(cursor==len(body),'no skipped original frame tail')
    framed=bytes(candidate);witness=bytes(witness)
    packed=zstd.ZstdCompressor(level=19,threads=0,write_checksum=True,write_content_size=True).compress(framed)
    total=summed(rows)
    summary=dict(index=index,file=name,source_commit=COMMIT,source_generator_sha256=GEN_SHA256,
        records=sum(r['records'] for r in rows),counts=total,directional=directional(total),
        compressed_sha256=sha(packed),compressed_bytes=len(packed),raw_record_sha256=sha(framed[24:]),
        witness_sha256=sha(witness),witness_bytes=len(witness),per_step=rows,
        parent_compressed_sha256=sha(old),parent_metadata_sha256=sha(meta_raw),
        raw_ccx_saved=sum(r['raw_ccx_saved'] for r in rows),added_cx=sum(r['added_cx'] for r in rows),
        selected_runs=sum(r['selected_runs'] for r in rows),generator_reexecuted=False,full9024=False,
        whole_Q=None,whole_T=None)
    return packed,witness,summary

def main():
    need(not sys.flags.optimize,'optimized Python forbidden')
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--original-cache',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True)
    p.add_argument('--expected-cache',type=Path)
    p.add_argument('--index',type=int)
    a=p.parse_args()
    for folder in (a.original_cache,a.expected_cache):
        if folder is not None:
            need(folder.is_absolute() and folder.resolve()==folder and folder.is_dir(),'canonical input directory')
    need(a.output.is_absolute() and a.output.resolve()==a.output and
         a.output.parent.is_dir() and not a.output.exists(),'new output directory required')
    indices=list(range(36)) if a.index is None else [a.index]
    need(all(0<=i<36 for i in indices),'bounded shard selection')
    resource.setrlimit(resource.RLIMIT_AS,(768*1024**2,)*2)
    resource.setrlimit(resource.RLIMIT_CPU,(21600,)*2)
    resource.setrlimit(resource.RLIMIT_CORE,(0,0))
    def stop(sig,frame):
        raise TimeoutError('reproducer wall limit')
    signal.signal(signal.SIGALRM,stop);signal.alarm(23760)
    import zstandard as zstd
    need(zstd.__version__=='0.23.0' and zstd.backend=='cext','zstandard 0.23.0 C backend required')
    a.output.mkdir();results=[]
    for index in indices:
        packed,witness,row=reproduce(index,a.original_cache,zstd)
        if a.expected_cache is not None:
            expected=read(a.expected_cache/row['file'])
            need(expected==packed,'byte-identical expected candidate '+row['file'])
            row['expected_candidate_byte_identical']=True
        save(a.output/row['file'],packed)
        save(a.output/(row['file']+'.witness.bin'),witness)
        save(a.output/(row['file']+'.json'),encoded(row))
        results.append(row)
        print(json.dumps(dict(index=index,status='REPRODUCED',compressed_sha256=row['compressed_sha256'])),flush=True)
    receipt=dict(status='EXACT_PUBLIC_CACHE_MAXIMAL_REPRODUCED',source_commit=COMMIT,
        reproducer_sha256=sha(read(Path(__file__).resolve())),indices=indices,shards=results,
        all36=len(indices)==36,all1616=sum(len(r['per_step']) for r in results)==1616,
        expected_cache_matched=a.expected_cache is not None,generator_reexecuted=False,
        compiler_calls=0,native_calls=0,full9024=False,whole_Q=None,whole_T=None)
    save(a.output/'reproduction.json',encoded(receipt));signal.alarm(0)

if __name__=='__main__':
    main()

