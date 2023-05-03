use clap::Parser;
use proxy::{Proxy, ProxyType};

mod download;
mod proxy;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "./")]
    dir: String,

    #[arg(long)]
    rules: bool,

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

    #[arg(id = "type", value_enum, long, default_value = "xray")]
    t: ProxyType,

    #[arg(long)]
    upgrade: bool,

    #[arg(
        long,
        default_value = "https://api.github.com/repos/XTLS/Xray-core/releases/latest"
    )]
    release_url: String,

    #[arg(long, default_value = "Xray-windows-64.zip")]
    asset_name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut should_restart = false;
    let proxy = proxy::new(&args.t, &args.dir, &args.release_url, &args.asset_name);

    if args.rules {
        proxy::download_rules(&args.dir, &args.ip_url, &args.domain_url).await?;
        should_restart = true;
    }

    if args.upgrade {
        proxy.upgrade()?;
        should_restart = true;
    }

    if should_restart {
        proxy.restart()?;
    }

    Ok(())
}
