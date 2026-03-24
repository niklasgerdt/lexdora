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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_parse_org_create() {
        let args = vec!["dora-cli", "org", "create", "ACME", "--legal-entity-id", "123"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::Org { cmd } => match cmd {
                OrgCmd::Create { name, legal_entity_id } => {
                    assert_eq!(name, "ACME");
                    assert_eq!(legal_entity_id, Some("123".to_string()));
                }
                _ => panic!("Expected OrgCmd::Create"),
            },
            _ => panic!("Expected Commands::Org"),
        }
    }

    #[test]
    fn test_cli_parse_serve() {
        let args = vec!["dora-cli", "serve", "--bind", "127.0.0.1:8080"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::Serve { bind } => {
                assert_eq!(bind, Some("127.0.0.1:8080".to_string()));
            }
            _ => panic!("Expected Commands::Serve"),
        }
    }

    #[test]
    fn test_cli_parse_user_create() {
        let org_id = Uuid::new_v4();
        let org_id_str = org_id.to_string();
        let args = vec![
            "dora-cli",
            "user",
            "create",
            &org_id_str,
            "alice@example.com",
            "--full-name",
            "Alice Smith",
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::User { cmd } => match cmd {
                UserCmd::Create { organization_id, email, full_name } => {
                    assert_eq!(organization_id, org_id);
                    assert_eq!(email, "alice@example.com");
                    assert_eq!(full_name, Some("Alice Smith".to_string()));
                }
                _ => panic!("Expected UserCmd::Create"),
            },
            _ => panic!("Expected Commands::User"),
        }
    }
}
