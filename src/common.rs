use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

use itertools::Itertools;
use noodles::core::{Position, Region};
use noodles_bgzf::{self, VirtualPosition, io::Reader};
use noodles_csi::{binning_index::Index, io::IndexedReader};

#[derive(Debug, Clone)]
pub struct RegionString(pub(crate) Region);

impl RegionString {
    pub fn new(region: &str) -> Result<Self, Box<dyn Error>> {
        let rgn = match region
            .rsplit_once(':')
            .and_then(|(chrom, coords)| coords.split_once('-').map(|(st, end)| (chrom, st, end)))
        {
            Some((chrom, st, end)) => Region::new(
                chrom,
                Position::new(st.parse::<usize>()?).unwrap()..=Position::new(end.parse()?).unwrap(),
            ),
            None => todo!(),
        };
        Ok(RegionString(rgn))
    }

    pub fn make_windows(&self, window: usize) -> Vec<Region> {
        let (st, end) = match (self.0.start(), self.0.end()) {
            (
                std::ops::Bound::Included(st) | std::ops::Bound::Excluded(st),
                std::ops::Bound::Included(end) | std::ops::Bound::Excluded(end),
            ) => (st.get(), end.get()),
            _ => unreachable!(),
        };
        interval_into_windows(self.0.name().to_string(), end - st, window, st).collect()
    }
}

pub fn read_bed(infile: &str, window: usize) -> Result<Vec<Region>, Box<dyn Error>> {
    let reader = BufReader::new(File::open(infile)?);
    let mut regions = vec![];
    for line in reader.lines() {
        let line = line?;
        if let Some((chrom, st, end)) = line.split('\t').take(3).collect_tuple() {
            let st = st.parse::<usize>()?.clamp(1, usize::MAX);
            let end: usize = end.parse()?;
            let length = end - st;
            regions.extend(interval_into_windows(chrom.to_owned(), length, window, st));
        }
    }
    Ok(regions)
}

pub fn interval_into_windows(
    chrom: String,
    length: usize,
    window: usize,
    offset: usize,
) -> impl Iterator<Item = Region> {
    let n_intervals = length / window;
    let length_equal_intervals = n_intervals * window;
    let remainder = length - length_equal_intervals;
    let final_region = Region::new(
        chrom.clone(),
        Position::new(length_equal_intervals + offset).unwrap()
            ..=Position::new(length_equal_intervals + remainder + offset).unwrap(),
    );
    (0..n_intervals + 1)
        .map(move |i| {
            let st = Position::new((i * window).clamp(1, usize::MAX) + offset).unwrap();
            let end = Position::new(((i + 1) * window) + offset).unwrap();
            Region::new(chrom.clone(), st..=end)
        })
        .chain(std::iter::once(final_region))
}

pub fn read_chrom_length_windows(
    infile: &str,
    window: usize,
) -> Result<Vec<Region>, Box<dyn Error>> {
    let chrom_lengths_fh = BufReader::new(File::open(infile)?);
    Ok(chrom_lengths_fh
        .lines()
        .map_while(Result::ok)
        .flat_map(|line| {
            let (chrom, length) = {
                let (chrom, length) = line.trim().split('\t').take(2).collect_tuple().unwrap();
                (chrom.to_owned(), length.parse::<usize>().unwrap())
            };
            interval_into_windows(chrom, length, window, 0)
        })
        .collect())
}

pub fn get_average_in_window(
    reader: &mut IndexedReader<Reader<File>, Index<Vec<VirtualPosition>>>,
    region: &Region,
) -> Result<f32, Box<dyn Error>> {
    let mut acc: f32 = 0.0;
    let mut n: usize = 0;
    let query = reader.query(region)?;
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
