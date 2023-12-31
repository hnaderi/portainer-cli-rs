use clap::{arg, Command, Arg};
use clap::{ArgGroup, ArgMatches};

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
        server: ServerConfig,
        stacks: Vec<String>,
        endpoint: EndpointSelector,
        confirmed: bool,
        configs: Vec<String>,
        secrets: Vec<String>,
    },
    Login {
        server: String,
        address: String,
        credential: LoginCredential,
    },
    Logout(String),
}

pub struct InlineEnv(pub String, pub String);
pub struct FileMapping(pub String, pub Box<Path>);

pub enum EndpointSelector {
    ByName(String),
    ById(i32),
    ByTags(Vec<String>),
    ByTagIds(Vec<i32>),
}

pub enum LoginCredential {
    ByUserPass { username: String, password: String },
    ByAPIToken(String),
}

pub enum ServerConfig {
    InlineToken {
        address: String,
        token: String,
    },
    InlineLogin {
        address: String,
        username: String,
        password: Option<String>,
    },
    Session(String),
}

type ParseResult<T> = Result<T, String>;

fn server_config_parse(matches: &ArgMatches) -> ParseResult<ServerConfig> {
    if let Some(session) = matches.get_one::<String>("session") {
        Ok(ServerConfig::Session(session.to_string()))
    } else {
        let address = matches.get_one::<String>("address");
        let token = matches.get_one::<String>("token");

        let username = matches.get_one::<String>("username");
        token
            .zip(address)
            .map(|(token, address)| ServerConfig::InlineToken {
                address: address.to_string(),
                token: token.to_string(),
            })
            .or_else(|| {
                address
                    .zip(username)
                    .map(|(address, username)| -> ServerConfig {
                        ServerConfig::InlineLogin {
                            address: address.to_string(),
                            username: username.to_string(),
                            password: matches.get_one::<String>("password").cloned(),
                        }
                    })
            })
            .ok_or_else(|| "You must enter either session or address and token".to_string())
    }
}

fn deploy_command() -> Command {
    app_args(
        Command::new("deploy").about("deploys stack and its dependencies")
        .arg(arg!(-f --compose <FILE> "compose file to deploy").required(true))
        .arg(arg!(--config <FILEMAPPING> "file mapping to be created as docker config, format `name:file`"))
        .arg(arg!(--secret <FILEMAPPING> "file mapping to be created as docker secret, format `name:file`"))
        .arg(arg!(-e --env <ENVVAR> "environment variables to add to stack, format `KEY=VALUE`, these take precedence over envfile"))
        .arg(arg!(--envfile <FILE> "dotenv file to add to stack, values are merged with other inline vars"))
        .arg(arg!(-Y --confirm "confirms automatically and do not ask for prompts"))
    )
}

fn server_config_args(cmd: Command) -> Command {
    let session = ArgGroup::new("from-session")
        .arg("session")
        .requires("session")
        .conflicts_with_all(["from-token", "from-userpass"]);
    let token = ArgGroup::new("from-token")
        .arg("token")
        .requires_all(["address", "token"])
        .conflicts_with_all(["from-session", "from-userpass"]);
    let userpass = ArgGroup::new("from-userpass")
        .arg("username")
        .requires_all(["address", "username"])
        .conflicts_with_all(["from-token", "from-session"]);

    cmd.arg(arg!(--token <token> "API token"))
        .arg(arg!(-u --username <username> "username to login"))
        .arg(arg!(-p --password <password> "password for login"))
        .arg(arg!(-H --address <url> "Server address"))
        .arg(arg!(-S --session <name> "Existing session name"))
        .groups([session, token, userpass])
}

fn endpoint_args(cmd: Command) -> Command {
    let endpoint = ArgGroup::new("endpoint-selector")
        .args(["name", "id", "tag", "tagid"])
        .required(true);
    let arg= Arg::new("").value_parser(parser);

    cmd.arg(arg!(-N --name <ENDPOINT> "endpoint name"))
        .arg(arg!(-E --id <ENDPOINT_ID> "endpoint id"))
        .arg(arg!(-t --tag <TAG> "endpoint tag"))
        .arg(arg!(-T --tagid <TAG_ID> "endpoint tag id"))
        .group(endpoint)
}

fn app_args(cmd: Command) -> Command {
    server_config_args(endpoint_args(cmd))
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
    app_args(Command::new("destroy").about("destroy stacks, configs, secrets"))
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
    match build_command().get_matches().subcommand() {
        Some(("deploy", matches)) => deploy_parse(&matches),
        Some(("destroy", matches)) => deploy_parse(&matches),
        Some(("login", matches)) => deploy_parse(&matches),
        Some(("logout", matches)) => deploy_parse(&matches),
        Some((cmd, _)) => ParseResult::Err(format!("Unknown command '{}'", cmd)),
        None => {
            build_command().print_help().expect("cannot print help");

            ParseResult::Err(format!("Command is required!"))
        }
    }
}
