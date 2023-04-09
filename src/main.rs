use clap::Parser;
use download::Downloader;
mod download;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = ".")]
    output: String,

    #[arg(
        short,
        long,
        default_value = "https://cdn.jsdelivr.net/gh/Loyalsoldier/v2ray-rules-dat@release/geoip.dat"
    )]
    ip_url: String,

    #[arg(
        short,
        long,
        default_value = "https://cdn.jsdelivr.net/gh/Loyalsoldier/v2ray-rules-dat@release/geosite.dat"
    )]
    domain_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let download = Downloader::new(&args.output);
    download.download(&args.ip_url).await?;
    download.download(&args.domain_url).await?;

    Ok(())
}
