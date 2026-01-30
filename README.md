# Pileup window comparer (pwc)
Compare pileups over windows.

## Why?
* Using interval intersection tools like `bedtools` requires loading every interval into memory when comparing two pileups.
* This does not scale well with `modkit` pileups and can reach 100s of GB with high coverage data. 
* We use `noodles` and `tabix` to query in a fast and memory efficient (disk-backed) manner.

## Usage
Compile with `cargo`.
```bash
cargo build --release
/target/release/pwc -h
```

Input is treatment (`-t`) and control (`-c`) pileups. Lengths (`-l`) and window (`-w`) determine output intervals. Mode (`-m`) is ratio (`t / c`) or diff (`t - c`); infinite values in `-m ratio` are skipped.
```bash
/target/release/pwc \
-t /project/logsdon_shared/projects/PrimateT2T/CenPlot/data/methylbed/mPanPan1_CENP-A_dimelo2matpat_v1.0.8.bed.gz \
-c /project/logsdon_shared/projects/PrimateT2T/CenPlot/data/methylbed/mPanPan1_noAb_dimelo2matpat_v1.0.8.bed.gz \
-l /project/logsdon_shared/data/PrimateT2T/assemblies/mPanPan1.matpat.v1.fa.fai \
-w 5000 \
-m ratio
```

Output is an unsorted BED4 file.
```
chr1_mat_hsa1   3615000 3620000 1.4033998
```

## TODO
* [ ] - Figure out if lengths can be removed. Look into `tabix` spec.
