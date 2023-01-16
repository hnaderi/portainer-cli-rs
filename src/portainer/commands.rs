use clap::ArgMatches;
use clap::{arg, Command};
use std::path::Path;

pub enum CLICommand {
    Deploy {
        server: ServerConfig,
        compose: String,
        stack: String,
        endpoint: EndpointSelector,
        confirmed: bool,
        inline_vars: Vec<InlineEnv>,
        configs: Vec<FileMapping>,
        secrets: Vec<FileMapping>,
    },
    Destroy {
        // server: ServerConfig,
        stack: String,
        // endpoint: EndpointSelector,
        confirmed: bool,
        configs: Vec<String>,
        secrets: Vec<String>,
    },
    Login {
        server: String,
        address: String,
        credential: LoginCredential,
    },
    Logout {
        server: String,
    },
}

pub struct InlineEnv {
    key: String,
    value: String,
}

pub struct FileMapping {
    name: String,
    source: Box<Path>,
}

pub enum EndpointSelector {
    ByName(String),
    ById(u32),
    ByTags(Vec<String>),
    ByTagIds(Vec<u32>),
}

pub enum LoginCredential {
    ByUserPass { username: String, password: String },
    ByAPIToken(String),
}

pub enum ServerConfig {
    Inline { address: String, token: String },
    Session(String),
}

type ParseResult<T> = Result<T, String>;

fn server_config_parse(matches: &ArgMatches) -> ParseResult<ServerConfig> {
    if let Some(session) = matches.get_one::<String>("session") {
        Ok(ServerConfig::Session(session.to_string()))
    } else {
        let address = matches.get_one::<String>("address");
        let token = matches.get_one::<String>("token");
        token
            .zip(address)
            .map(|(token, address)| ServerConfig::Inline {
                address: address.to_string(),
                token: token.to_string(),
            })
            .ok_or_else(|| "You must enter either session or address and token".to_string())
    }
}
fn deploy_command() -> Command {
    Command::new("deploy")
        .about("deploys stack and its dependencies")
        .arg(arg!(--token <token> "API token"))
        .arg(arg!(--address <url> "Server address"))
        .arg(arg!(--session <name> "Existing session name"))
}

fn deploy_parse(matches: &ArgMatches) -> ParseResult<CLICommand> {
    Ok(CLICommand::Deploy {
        server: server_config_parse(matches)?,
        compose: todo!(),
        stack: todo!(),
        endpoint: todo!(),
        confirmed: todo!(),
        inline_vars: todo!(),
        configs: todo!(),
        secrets: todo!(),
    })
}
fn destroy_command() -> Command {
    Command::new("destroy").about("destroy stacks, configs, secrets")
}
fn login_command() -> Command {
    Command::new("login").about("login to server and adds it to sessions")
}
fn logout_command() -> Command {
    Command::new("logout").about("removes server from logged in sessions")
}

fn build_command() -> Command {
    Command::new("pctl")
        .author("Hossein Naderi <mail@hnaderi.dev>")
        .version("0.1.0")
        .about("Save human time by using this client to automate workflows in CI/CD or other pipelines.")
        .color(clap::ColorChoice::Auto)
        .subcommand(deploy_command())
        .subcommand(destroy_command())
        .subcommand(login_command())
        .subcommand(logout_command())
}
pub fn parse_command() -> ParseResult<CLICommand> {
    let matches = build_command().get_matches();
    match matches.subcommand() {
        Some(("deploy", matches)) => deploy_parse(&matches),
        Some(("destroy", matches)) => deploy_parse(&matches),
        Some(_value) => todo!(),
        None => todo!(),
    }
}
