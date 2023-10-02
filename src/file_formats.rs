pub enum FileFormat {
    Ansi,
    Txt,
    Dur,
}

impl FileFormat {
    pub fn from_extension(extension: Option<&str>) -> Result<FileFormat, ()> {
        match extension {
            Some("txt") => Ok(FileFormat::Txt),
            Some("ansi") => Ok(FileFormat::Ansi),
            Some("ans") => Ok(FileFormat::Ansi),
            Some("dur") => Ok(FileFormat::Dur),
            Some(_) => Err(()),
            None => Ok(FileFormat::Ansi),
        }
    }
}
