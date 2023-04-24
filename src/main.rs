use clap::Parser;
use download::Downloader;
mod download;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "./")]
    output: String,

    #[arg(
        long,
        default_value = "https://cdn.jsdelivr.net/gh/Loyalsoldier/v2ray-rules-dat@release/geoip.dat"
    )]
    ip_url: String,

    #[arg(
        long,
        default_value = "https://cdn.jsdelivr.net/gh/Loyalsoldier/v2ray-rules-dat@release/geosite.dat"
    )]
    domain_url: String,

    #[arg(long, default_value = "wxray")]
    proxy_name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    download_rules(&args).await?;
    restart(&args.proxy_name);

    Ok(())
}

async fn download_rules(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let downloader = Downloader::new(&args.output);
    println!("{}: Downloading geoip.dat...", chrono::Local::now());
    downloader.download(&args.ip_url).await?;
    println!("{}: Downloading geosite.dat...", chrono::Local::now());
    downloader.download(&args.domain_url).await?;

    Ok(())
}

#[cfg(target_os = "windows")]
fn restart(process_name: &str) {
    let mut cmd = std::process::Command::new("taskkill");
    cmd.arg("/F")
        .arg("/IM")
        .arg(process_name.to_string() + ".exe");
    cmd.output().expect("failed to execute process");
    std::process::Command::new(process_name)
        .spawn()
        .expect("failed to execute process");
}

#[cfg(not(target_os = "windows"))]
fn restart(process_name: &str) {
    let mut cmd = std::process::Command::new("pkill");
    cmd.arg(process_name);
    cmd.output().expect("failed to execute process");
    std::process::Command::new(process_name)
        .spawn()
        .expect("failed to execute process");
}
