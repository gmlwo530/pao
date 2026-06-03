use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "pao", version, about = "Project Agent Orchestrator")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Initialize a PAO workspace in the current directory.
    Init,
    /// Manage repositories registered in the current workspace.
    Repo {
        #[command(subcommand)]
        command: RepoCommand,
    },
    /// Synchronize registered repository metadata.
    Sync,
    /// Manage workspace tasks.
    Task {
        #[command(subcommand)]
        command: TaskCommand,
    },
    /// Start an AI client session for a registered repository.
    Chat(ChatArgs),
    /// Manage local AI client commands.
    Client {
        #[command(subcommand)]
        command: ClientCommand,
    },
    /// Print local diagnostics.
    Doctor,
}

#[derive(Debug, Subcommand)]
pub enum RepoCommand {
    /// Register a repository in the current workspace.
    Add(RepoAddArgs),
    /// Remove a repository from the current workspace.
    Remove(RepoRemoveArgs),
    /// List registered repositories.
    List,
    /// Show status for registered repositories.
    Status(RepoStatusArgs),
}

#[derive(Debug, Args)]
pub struct RepoAddArgs {
    pub name: String,
    #[arg(long)]
    pub remote: String,
    #[arg(long)]
    pub branch: String,
}

#[derive(Debug, Args)]
pub struct RepoRemoveArgs {
    pub name: String,
    #[arg(long, required = true)]
    pub keep_checkout: bool,
}

#[derive(Debug, Args)]
pub struct RepoStatusArgs {
    pub name: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum TaskCommand {
    /// Create a task in the current workspace.
    Create(TaskCreateArgs),
}

#[derive(Debug, Args)]
pub struct TaskCreateArgs {
    pub task_id: String,
}

#[derive(Debug, Args)]
pub struct ChatArgs {
    #[arg(long)]
    pub repo: Option<String>,
    #[arg(long)]
    pub task: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum ClientCommand {
    /// Register a local AI client command.
    Add(ClientAddArgs),
    /// List registered local AI clients.
    List,
    /// Set the default local AI client.
    SetDefault(ClientSetDefaultArgs),
}

#[derive(Debug, Args)]
pub struct ClientAddArgs {
    pub name: String,
    #[arg(long)]
    pub command: String,
}

#[derive(Debug, Args)]
pub struct ClientSetDefaultArgs {
    pub name: String,
}
