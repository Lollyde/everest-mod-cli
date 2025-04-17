use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = "Mod management tool for Celeste", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Search for mods
    Search(SearchArgs),
    /// Show mod information from the remote catalog
    Info(InfoArgs),
    /// Install a mod
    Install(InstallArgs),
    /// List installed mods
    List,
    /// Show detailed information about an installed mod
    Show(ShowArgs),
    /// Check for updates
    Update(UpdateArgs),
}

#[derive(Args)]
pub struct SearchArgs {
    /// Search query
    pub query: String,
}

#[derive(Args)]
pub struct InfoArgs {
    /// Mod name
    pub name: String,
}

#[derive(Args)]
pub struct InstallArgs {
    /// Mod name
    pub name: String,
}

#[derive(Args)]
pub struct ShowArgs {
    /// Mod name
    pub name: String,
}

#[derive(Args)]
pub struct UpdateArgs {
    /// Install available updates
    #[arg(long, action)]
    pub install: bool,
}
