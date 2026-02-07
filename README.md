# (W)indow (A)ggregated (B)edgraph
Get windowed aggregated Bedgraphs.

## Why?
* Using interval intersection tools like `bedtools` requires loading every interval into memory when comparing two pileups.
* This does not scale well with `modkit` pileups and can reach 100s of GB.
* We use `noodles` and `tabix` to query in a fast and memory efficient (disk-backed) manner.

## Usage
Compile with `cargo`.
```bash
cargo build --release
/target/release/wab -h
```

### `window`
Input (`-i`) is the `modkit` pileup. Specify a region with `-r` as either a region string `chr1:1-100`, a BED3 file, or with chrom lengths (`fai`)
```bash
/target/release/wab window \
-i test/input/mGorGor1.matpat.v2_methyl.bed.gz \
-r test/input/mGorGor1.matpat.v2.centromeric_regions.bed | \
sort -k1,1 -k2,2n > out.bedgraph
```

### `paired`
Input is treatment (`-t`) and control (`-c`) pileups. Lengths (`-l`) and window (`-w`) determine output intervals. Mode (`-m`) is ratio (`t / c`) or diff (`t - c`); infinite values in `-m ratio` are skipped.
```bash
/target/release/wab paired \
-t /project/logsdon_shared/projects/PrimateT2T/CenPlot/data/methylbed/mPanPan1_CENP-A_dimelo2matpat_v1.0.8.bed.gz \
-c /project/logsdon_shared/projects/PrimateT2T/CenPlot/data/methylbed/mPanPan1_noAb_dimelo2matpat_v1.0.8.bed.gz \
-l /project/logsdon_shared/data/PrimateT2T/assemblies/mPanPan1.matpat.v1.fa.fai \
-w 5000 \
-m ratio | \
sort -k1,1 -k2,2n > out.bedgraph
```

Output is an unsorted BED4 file so we sort after.
```
chr1_mat_hsa1   3615000 3620000 1.4033998
```

## Benchmark
Using `test/script/calculate_windows.py`:
```bash
# Python
time python test/script/calculate_windows.py -b test/input/mGorGor1.matpat.v2.centromeric_regions.bed -m test/input/mGorGor1.matpat.v2_methyl.bed.gz -p 4 >
py_out_windows.bedgraph
# wab
time target/release/wab window \
-b test/input/mGorGor1.matpat.v2.centromeric_regions.bed \
-i test/input/mGorGor1.matpat.v2_methyl.bed.gz \
-t 4 | sort -k1,1 -k2,2n > wab_out_windows.bedgraph
```

While we see decent runtime improvements, the major advantage is memory usage.
```
# Python
real    2m44.335s
user    5m28.222s
sys     0m15.969s
# wab
real    1m29.474s
user    5m21.954s
sys     0m14.096s
```

## TODO
* [ ] - Figure out if lengths can be removed. Look into `tabix` spec.
