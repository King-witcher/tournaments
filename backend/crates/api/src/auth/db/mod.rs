//! Todo o SQL do domínio, dividido por conceito.

mod deletion;
mod emails;
mod logins;
mod passwords;
mod users;

pub use deletion::*;
pub use emails::*;
pub use logins::*;
pub use passwords::*;
pub use users::*;
