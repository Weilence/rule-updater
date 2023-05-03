use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::ValueEnum;
use regex::Regex;
use semver::Version;
use serde::Deserialize;

use crate::download::Downloader;

pub trait Proxy {
    fn version(&self) -> Result<Version, Box<dyn Error>>;
    fn restart(&self) -> Result<(), Box<dyn Error>>;
    fn upgrade(&self) -> Result<(), Box<dyn Error>>;
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ProxyType {
    Xray,
    V2ray,
}

pub fn new(t: &ProxyType, dir: impl AsRef<Path>, url: &str, asset_name: &str) -> impl Proxy {
    match t {
        ProxyType::Xray => Xray::new(dir, url, asset_name),
        _ => panic!("unsupported proxy type: {:?}", t),
    }
}

pub async fn download_rules(
    dir: &str,
    ip_url: &str,
    domain_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let downloader = Downloader::new();
    println!("{}: Downloading geoip.dat...", chrono::Local::now());
    downloader.download(ip_url, dir).await?;
    println!("{}: Downloading geosite.dat...", chrono::Local::now());
    downloader.download(domain_url, dir).await?;

    Ok(())
}

pub struct Xray {
    name: String,
    daemon_name: String,
    dir: PathBuf,
    latest_url: String,
    asset_name: String,
}

impl Xray {
    fn new(dir: impl AsRef<Path>, url: &str, asset_name: &str) -> Self {
        let latest_url = url.to_string();
        let asset_name = asset_name.to_string();
        let mut name = "xray".to_string();
        let mut daemon_name = "wxray".to_string();

        if cfg!(target_os = "windows") {
            name = name + ".exe";
            daemon_name = daemon_name + ".exe";
        }

        Xray {
            name,
            daemon_name,
            latest_url,
            asset_name,
            dir: dir.as_ref().to_path_buf(),
        }
    }
}

impl Proxy for Xray {
    fn version(&self) -> Result<Version, Box<dyn Error>> {
        let mut cmd = std::process::Command::new(&self.name);
        cmd.current_dir(&self.dir);
        cmd.arg("version");
        let output = cmd.output();

        let output = match output {
            Ok(output) => output,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    return Ok(Version::new(0, 0, 0));
                } else {
                    return Err(Box::new(err));
                }
            }
        };

        let output_text = String::from_utf8(output.stdout)?;
        let re = Regex::new(r"(\d+\.\d+\.\d+)")?;

        let versionstr = re
            .captures(&output_text)
            .ok_or("failed to parse version string from output of xray version command")
            .unwrap()[1]
            .to_string();

        let version = Version::parse(&versionstr)?;

        Ok(version)
    }

    fn restart(&self) -> Result<(), Box<dyn Error>> {
        let mut cmd: Command;
        if cfg!(target_os = "windows") {
            cmd = Command::new("taskkill");
            cmd.arg("/F").arg("/IM").arg(&self.daemon_name);
        } else {
            cmd = Command::new("pkill");
            cmd.arg(&self.daemon_name);
        };

        cmd.current_dir(&self.dir);
        cmd.output()?;

        std::process::Command::new(&self.daemon_name).spawn()?;

        Ok(())
    }

    fn upgrade(&self) -> Result<(), Box<dyn Error>> {
        let current_version = self.version()?;
        let latest_release = futures::executor::block_on(Release::from_url(&self.latest_url))?;
        let latest_version = latest_release.version();

        if current_version >= latest_version {
            println!("{}: Already latest version.", chrono::Local::now());
            return Ok(());
        }

        println!(
            "{}: New version {} is available!",
            chrono::Local::now(),
            latest_version
        );

        let downloader = Downloader::new();

        println!(
            "{}: Downloading {}...",
            chrono::Local::now(),
            &self.asset_name
        );

        let asset_url = latest_release.asset_url(&self.asset_name);
        let zip_file = futures::executor::block_on(
            downloader.download(&asset_url, self.dir.to_str().unwrap()),
        )?;

        println!("{}: Unzipping...", chrono::Local::now());

        let mut zip = zip::ZipArchive::new(std::fs::File::open(zip_file)?)?;
        zip.extract(&self.dir)?;

        Ok(())
    }
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

impl Release {
    async fn from_url(url: &str) -> Result<Release, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let release = client
            .get(url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "Xray-Proxy-Updater")
            .send()
            .await?
            .json()
            .await?;

        Ok(release)
    }

    fn version(&self) -> Version {
        Version::parse(&self.tag_name.replace("v", "")).unwrap()
    }

    fn asset_url(&self, asset_name: &str) -> String {
        for asset in &self.assets {
            if asset.name == asset_name {
                return asset.browser_download_url.clone();
            }
        }

        panic!("Asset not found!");
    }
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}
