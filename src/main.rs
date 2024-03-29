pub mod config;
pub mod client;
pub mod behaviour;
pub mod request_responce;

use behaviour::{Behaviour, Event};
use client::Client;
use config::Config;
use libc::user;
use request_responce::{PacketRequest, PacketStreamCodec, PacketStreamProtocol};

use clap::{ Parser, Subcommand};
use std::path::PathBuf;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Sync + Send>>;

pub(crate) const MTU: usize = 1420;

#[derive(Debug)]
enum Error {
    NoPeers,
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoPeers => fmt.write_str("No VPN peers specified in config"),
        }
    }
}

    impl std::error::Error for Error {}
    
    #[derive(Parser)]
    struct Opts {
        #[clap(subcommand)]
        command: CliCommand
    }


    #[derive(Subcommand)]
enum CliCommand {
    Init {
        #[clap(long, default_value = "/etc/p2p-tun/config.yaml")]
        config: String, 
    },
    Run {
        #[clap(long, default_value = "/etc/p2p-tun/config.yaml")]
        config: String
    }
}

fn main() -> Result<()> {
    env_logger::init();
    let opts = Opts::parse();
    match opts.command {
        CliCommand::Init { config } => {
            let cfg_path = PathBuf::from(config);
            let cfg = Config::default();
            std::fs::write(cfg_path, serde_yaml::to_string(&cfg)?)?;
        }
        CliCommand::Run { config } => {
            let cfg: Config = {
                let cfg_path = PathBuf::from(config);
                let cfg_str = std::fs::read_to_string(&cfg_path)?;
                let cfg = serde_yaml::from_str(&cfg_str)?;
                cfg
            };
            async_std::task::block_on(async move {
                log::debug!("Creating tun device...");
                let tun = async_tun::TunBuilder::new()
                    .name("")
                    .tap(false)
                    .mtu(MTU as i32)
                    .packet_info(false)
                    .up()
                    .try_build()
                    .await?;

                log::debug!("Swithing to user: {}", cfg.user());
                drop_privilieges(&cfg.user())?;
                log::debug!("Starting SWARM client");
                let mut client = Client::builder()
                    .config(cfg)
                    .tun(tun)
                    .build()
                    .await
                    .unwrap();
                client.run().await
            })?;
        }
    }
    Ok(())
}

fn drop_privilieges(username: &str) -> Result<()> {
    let uid = users::get_user_by_name(username)
        .expect("User to exist")
        .uid();
    let gid = users::get_group_by_name(username)
        .expect("Group to exist")
        .gid();
        users::switch::set_both_gid(gid, gid)?;
        users::switch::set_both_gid(uid, uid)?;

    Ok(())
}