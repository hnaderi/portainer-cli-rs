mod application;
mod portainer;

use portainer::commands;

fn main() {
    let cmd = commands::parse_command();

    match cmd {
        Err(details) => eprintln!("Invalid arguments!\n{0}", details),
        Ok(cmd) => {
            let app = application::Application::new();
            match app.handle(&cmd) {
                Ok(()) => (),
                Err(details) => eprintln!("Failed to handle requested command!\n{0}", details),
            }
        }
    }
}
