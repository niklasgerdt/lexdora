use clap::{Parser, Subcommand};
use uuid::Uuid;

pub mod handlers;

#[derive(Parser, Debug)]
#[command(name = "dora-cli", version, about = "Simple CRUD CLI for DORA schema (PostgreSQL)")]
pub struct Cli {
    /// Database URL, e.g. postgres://user:pass@localhost:5432/db
    #[arg(long)]
    pub database_url: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Org { #[command(subcommand)] cmd: OrgCmd },
    User { #[command(subcommand)] cmd: UserCmd },
    Incident { #[command(subcommand)] cmd: IncidentCmd },
    Tpp { #[command(subcommand)] cmd: TppCmd },
    /// Start HTTP REST server
    Serve { #[arg(long)] bind: Option<String> },
}

#[derive(Subcommand, Debug)]
pub enum OrgCmd {
    Create { name: String, #[arg(long)] legal_entity_id: Option<String> },
    List,
    Get { id: Uuid },
    Update { id: Uuid, #[arg(long)] name: Option<String>, #[arg(long)] legal_entity_id: Option<String> },
    Delete { id: Uuid },
}

#[derive(Subcommand, Debug)]
pub enum UserCmd {
    Create { organization_id: Uuid, email: String, #[arg(long)] full_name: Option<String> },
    List { #[arg(long)] organization_id: Option<Uuid> },
    Get { id: Uuid },
    Update { id: Uuid, #[arg(long)] email: Option<String>, #[arg(long)] full_name: Option<String>, #[arg(long)] is_active: Option<bool> },
    Delete { id: Uuid },
}

#[derive(Subcommand, Debug)]
pub enum IncidentCmd {
    Create {
        organization_id: Uuid,
        title: String,
        #[arg(long, value_name = "incident_type")] type_: String,
        #[arg(long, value_name = "incident_severity")] severity: String,
        #[arg(long, value_name = "RFC3339|now")] detected_at: String,
        #[arg(long)] description: Option<String>,
        #[arg(long)] is_major: Option<bool>,
    },
    List { #[arg(long)] organization_id: Option<Uuid> },
    Get { id: Uuid },
    Update {
        id: Uuid,
        #[arg(long)] title: Option<String>,
        #[arg(long, value_name = "incident_type")] type_: Option<String>,
        #[arg(long, value_name = "incident_severity")] severity: Option<String>,
        #[arg(long)] description: Option<String>,
        #[arg(long)] is_major: Option<bool>,
    },
    Delete { id: Uuid },
}

#[derive(Subcommand, Debug)]
pub enum TppCmd {
    Create { organization_id: Uuid, name: String, #[arg(long)] country: Option<String>, #[arg(long)] criticality: Option<String>, #[arg(long)] is_important: Option<bool> },
    List { #[arg(long)] organization_id: Option<Uuid> },
    Get { id: Uuid },
    Update { id: Uuid, #[arg(long)] name: Option<String>, #[arg(long)] country: Option<String>, #[arg(long)] criticality: Option<String>, #[arg(long)] is_important: Option<bool> },
    Delete { id: Uuid },
}
