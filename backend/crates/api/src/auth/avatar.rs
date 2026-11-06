//! O avatar que as telas mostram: a foto enviada (`users.avatar_key`, no
//! bucket público, que o domínio `images` grava) ou, sem ela, o do Discord
//! (`users.avatar_url`, que cada login pelo Discord atualiza). Toda query que
//! mostra uma pessoa lê `coalesce(u.avatar_key, u.avatar_url) as avatar_url`
//! e passa o valor por `resolve_avatar`: o banco garante que um é sempre
//! `avatars/...` e o outro `https://...`, então o prefixo diz qual veio.

use crate::shared::public_url;

const PHOTO_PREFIX: &str = "avatars/";

/// O `coalesce(u.avatar_key, u.avatar_url)` lido do banco como a URL que o
/// front mostra: o nome da foto vira o endereço no CDN (sem o CDN
/// configurado, nada); a URL do Discord passa como está.
pub fn resolve_avatar(stored: Option<String>) -> Option<String> {
    let stored = stored?;
    if stored.starts_with(PHOTO_PREFIX) {
        public_url(&stored)
    } else {
        Some(stored)
    }
}

/// `resolve_avatar` em cada item de uma lista lida com `query_as!`.
pub fn resolve_avatars<T>(rows: &mut [T], field: impl Fn(&mut T) -> &mut Option<String>) {
    for row in rows {
        let value = field(row);
        *value = resolve_avatar(value.take());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_discord_url_passes_through() {
        let url = "https://cdn.discordapp.com/avatars/1/a.png".to_owned();
        assert_eq!(resolve_avatar(Some(url.clone())), Some(url));
        assert_eq!(resolve_avatar(None), None);
    }
}
