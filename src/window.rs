use std::error::Error;

use noodles_tabix as tabix;
use rayon::{ThreadPoolBuilder, prelude::*};

use crate::{
    cli::WindowArgs,
    common::{RegionString, get_average_in_window, read_bed, read_chrom_length_windows},
};

pub fn window_pileup(args: WindowArgs) -> Result<(), Box<dyn Error>> {
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
        let mut reader = tabix::io::indexed_reader::Builder::default()
            .build_from_path(&args.infile)
            .unwrap();

        let avg = get_average_in_window(&mut reader, &region).unwrap();
        let st = match region.start() {
            std::ops::Bound::Included(v) | std::ops::Bound::Excluded(v) => v.get(),
            std::ops::Bound::Unbounded => unreachable!(),
        };
        let end = match region.end() {
            std::ops::Bound::Included(v) | std::ops::Bound::Excluded(v) => v.get(),
            std::ops::Bound::Unbounded => unreachable!(),
        };
        println!("{}\t{st}\t{end}\t{}", region.name(), avg)
    });

    Ok(())
}
