use std::path::Path;

#[derive(Debug, Default)]
pub enum FileFormat {
    #[default]
    Ansi,
    Txt,
}

impl FileFormat {
    pub fn from_extension(extension: Option<&str>) -> Result<FileFormat, ()> {
        match extension {
            Some(text) => match text.to_lowercase().as_str() {
                "ansi" => Ok(FileFormat::Ansi),
                "ans" => Ok(FileFormat::Ansi),
                "txt" => Ok(FileFormat::Txt),
                _ => Err(()),
            },
            None => Ok(FileFormat::default()),
        }
    }
}

impl TryFrom<&str> for FileFormat {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let path = Path::new(value);
        let extension = match path.extension() {
            Some(extension) => extension.to_str(),
            None => None,
        };
        let format = match FileFormat::from_extension(extension) {
            Ok(format) => format,
            Err(()) => {
                return match extension {
                    Some(extension) => Err(anyhow::anyhow!(
                        "File extension not recognized: .{extension}"
                    )),
                    None => Err(anyhow::anyhow!("No file extension provided")),
                }
            }
        };
        Ok(format)
    }
}
