//! Todo o SQL do domínio, dividido por conceito.

mod emails;
mod logins;
mod passwords;
mod users;

pub use emails::*;
pub use logins::*;
pub use passwords::*;
pub use users::*;
