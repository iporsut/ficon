#[macro_use]
extern crate serde_derive;
extern crate regex;
extern crate structopt;

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "ficon")]
pub struct CliOption {
    /// Path to directory to check convention
    #[structopt(name = "PATH", default_value = ".", parse(from_os_str))]
    pub path: PathBuf,
}

#[derive(Deserialize)]
pub struct Config {
    default: SubConfig,
    extension: Option<Vec<SubConfigWithExtension>>,
}

#[derive(Deserialize)]
struct SubConfig {
    convention: String,
}

#[derive(Deserialize, Debug)]
struct SubConfigWithExtension {
    extension: String,
    convention: String,
}

pub struct Ficon {
    option: CliOption,
    config: Config,
}

impl Ficon {
    pub fn new() -> Ficon {
        let option: CliOption = CliOption::from_args();

        let config_path = if option.path.is_dir() {
            format!("{}/{}", option.path.display(), "ficon.toml")
        } else {
            panic!("path specified is not a directory")
        };

        let config = fs::read_to_string(&config_path)
            .expect(format!("can't read file from the path specified: {}", config_path).as_str());

        let config: Config = toml::from_str(config.as_str()).unwrap();

        return Ficon { option, config };
    }

    pub fn target_dir(&self) -> &Path {
        return self.option.path.as_ref();
    }

    pub fn check(&self, path: &Path) -> bool {
        let convention = self.config.convention_for(path);

        let pattern = match convention.as_str() {
            "kebab" => Regex::new(r"^[a-z][a-z\-]*[a-z]$").unwrap(),
            "snake" => Regex::new(r"^[a-z][a-z_]*[a-z]$").unwrap(),
            "upper_snake" => Regex::new(r"^[A-Z][A-Z_]*$").unwrap(),
            "camel" => Regex::new(r"^[a-z][A-Za-z]*$").unwrap(),
            "pascal" => Regex::new(r"^[A-Z][A-Za-z]*$").unwrap(),
            // TODO:
            // underscore_pre
            // underscore_post
            // underscore_surround
            _ => panic!("case not found {}", convention),
        };

        pattern.is_match(
            path.file_stem()
                .expect("file stem is missing")
                .to_str()
                .expect("can't cast file stem to string"),
        )
    }
}

impl Config {
    fn convention_for(&self, path: &Path) -> String {
        let extensions = &self.extension;

        let empty_vec = vec![];
        let extensions = extensions.as_ref().map_or(&empty_vec, |e| e);

        let matched_formats: Vec<&SubConfigWithExtension> = extensions
            .iter()
            .filter(|e| {
                e.extension
                    == path
                        .extension()
                        .unwrap_or(OsStr::new(""))
                        .to_str()
                        .unwrap_or("")
            })
            .collect();

        return matched_formats
            .first()
            .map(|e| e.convention.clone())
            .unwrap_or(self.default.convention.clone());
    }
}
