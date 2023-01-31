pub mod api;
pub mod client;
pub mod commands;
pub mod requests;
pub mod session;

type Res<T> = Result<T, String>;
type Action = Res<()>;
