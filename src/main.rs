use itertools::Itertools;
use noodles::core::{Position, Region};
use noodles_bgzf::{self, VirtualPosition, io::Reader};
use noodles_csi::{
    binning_index::Index,
    io::IndexedReader,
};
use noodles_tabix as tabix;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    error::Error, fs::File, io::{BufRead, BufReader}
};
use clap::{Parser, ValueEnum};

mod cli;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Mode {
    Diff,
    Ratio,
}

fn get_average_in_window(
    reader: &mut IndexedReader<Reader<File>, Index<Vec<VirtualPosition>>>,
    region: &Region,
) -> Result<f32, Box<dyn Error>> {
    let mut acc: f32 = 0.0;
    let mut n: usize = 0;
    let query = reader.query(&region)?;
    for rec in query {
        let rec = rec?;
        let (_chrom, _st, _end, _modification, _, _strand, _tst, _tend, _item_rgb, _, avg_percent) =
            rec.as_ref().split('\t').take(11).collect_tuple().unwrap();
        if avg_percent == "nan" {
            continue;
        }
        let avg_percent: f32 = avg_percent.parse()?;
        acc += avg_percent;
        n += 1;
    }
    Ok(acc / n as f32)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::Args::parse();

    let chrom_lengths_fh = BufReader::new(File::open(args.lengths_chrom)?);
    // TODO: As far as I can tell, there exists no way to extract the total length of a given chromosome from a tabix index.
    // If you do find a way, the fai argument can be removed.
    let chrom_windows: Vec<Region> = chrom_lengths_fh
        .lines()
        .map_while(Result::ok)
        .flat_map(|line| {
            let (chrom, n_intervals) = {
                let (chrom, length) = line.trim().split('\t').take(2).collect_tuple().unwrap();
                let length = length.parse::<usize>().unwrap();
                let n_intervals = length / args.window;
                (chrom.to_owned(), n_intervals)
            };
            (0..n_intervals + 1)
                .map(move |i| {
                    let st = Position::new((i * args.window).clamp(1, usize::MAX)).unwrap();
                    let end = Position::new((i + 1) * args.window).unwrap();
                    Region::new(chrom.clone(), st..=end)
                })
                .into_iter()
        })
        .collect();

    chrom_windows.into_par_iter().for_each(|region| {
        let mut control_reader =
            tabix::io::indexed_reader::Builder::default().build_from_path(&args.control).unwrap();
        let mut treatment_reader =
            tabix::io::indexed_reader::Builder::default().build_from_path(&args.treatment).unwrap();

        let treatment_avg = get_average_in_window(&mut control_reader, &region).unwrap();
        let control_avg = get_average_in_window(&mut treatment_reader, &region).unwrap();
        
        let chrom = region.name();
        let st = match region.start() {
            std::ops::Bound::Included(v) | std::ops::Bound::Excluded(v) => v.get(),
            std::ops::Bound::Unbounded => unreachable!(),
        };
        let end = match region.end() {
            std::ops::Bound::Included(v) | std::ops::Bound::Excluded(v) => v.get(),
            std::ops::Bound::Unbounded => unreachable!(),
        };
        let value = match args.mode {
            Mode::Diff => treatment_avg - control_avg,
            Mode::Ratio => {
                let value = treatment_avg / control_avg;
                if value.is_nan() {
                    0.0
                } else {
                    value
                }
            },
        };
        println!("{chrom}\t{st}\t{end}\t{value}")
    });
    Ok(())
}
