use clap::ValueEnum;
use noodles_tabix as tabix;
use rayon::{
    ThreadPoolBuilder,
    iter::{IntoParallelIterator, ParallelIterator},
};
use std::error::Error;

use crate::{
    cli::PairArgs,
    common::{RegionString, get_average_in_window, read_bed, read_chrom_length_windows},
};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum Mode {
    Diff,
    Ratio,
}

pub fn pair_pileups(args: PairArgs) -> Result<(), Box<dyn Error>> {
    ThreadPoolBuilder::new()
        .num_threads(args.threads)
        .build_global()?;

    let over = args.over;
    let regions = if let Some(bed) = over.bed {
        read_bed(bed.to_str().unwrap(), args.window)?
    } else if let Some(region) = over.region {
        let rgn = RegionString::new(&region)?;
        rgn.make_windows(args.window)
    } else if let Some(lengths_chrom) = over.lengths_chrom {
        read_chrom_length_windows(lengths_chrom.to_str().unwrap(), args.window)?
    } else {
        panic!("No valid regions provided.");
    };

    regions.into_par_iter().for_each(|region| {
        let mut control_reader = tabix::io::indexed_reader::Builder::default()
            .build_from_path(&args.control)
            .unwrap();
        let mut treatment_reader = tabix::io::indexed_reader::Builder::default()
            .build_from_path(&args.treatment)
            .unwrap();

        let (Ok(treatment_avg), Ok(control_avg)) = (
            get_average_in_window(&mut control_reader, &region),
            get_average_in_window(&mut treatment_reader, &region),
        ) else {
            eprintln!("Region {region:?} failed for either control or treatment.");
            return;
        };

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
                if value.is_nan() { 0.0 } else { value }
            }
        };
        println!("{chrom}\t{st}\t{end}\t{value}")
    });
    Ok(())
}
