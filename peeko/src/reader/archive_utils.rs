use anyhow;
use flate2::read::GzDecoder;
use std::{
    fs::File,
    io::{BufReader, Read},
};
use tar::Archive;
use zstd;

use std::path::Path;

pub(crate) fn read_tar_file<P: AsRef<Path>>(
    archive_path: P,
) -> anyhow::Result<Archive<Box<dyn Read>>> {
    let file = File::open(archive_path)?;
    let reader = BufReader::new(file);
    Ok(Archive::new(Box::new(reader)))
}

pub(crate) fn read_gzip_file<P: AsRef<Path>>(
    archive_path: P,
) -> anyhow::Result<Archive<Box<dyn Read>>> {
    let file = File::open(archive_path)?;
    let decoder = GzDecoder::new(file);
    Ok(Archive::new(Box::new(decoder)))
}

pub(crate) fn read_zstd_file<P: AsRef<Path>>(
    archive_path: P,
) -> anyhow::Result<Archive<Box<dyn Read>>> {
    let file = File::open(archive_path)?;
    let decoder = zstd::Decoder::new(file)?;
    Ok(Archive::new(Box::new(decoder)))
}
