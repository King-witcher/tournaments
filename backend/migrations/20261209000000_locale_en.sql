-- Terceiro idioma do site: inglês. Os cinco checks de `locale` passam a aceitar
-- 'en' (mesmos nomes de antes).
alter table users drop constraint users_locale_check;
alter table users add constraint users_locale_check check (locale in ('pt', 'es', 'en'));

alter table tournaments drop constraint tournaments_locale_check;
alter table tournaments add constraint tournaments_locale_check check (locale in ('pt', 'es', 'en'));

alter table tournament_translations drop constraint tournament_translations_locale_check;
alter table tournament_translations add constraint tournament_translations_locale_check check (locale in ('pt', 'es', 'en'));

alter table push_subscriptions drop constraint push_subscriptions_locale_check;
alter table push_subscriptions add constraint push_subscriptions_locale_check check (locale in ('pt', 'es', 'en'));

alter table discord_webhooks drop constraint discord_webhooks_locale_check;
alter table discord_webhooks add constraint discord_webhooks_locale_check check (locale in ('pt', 'es', 'en'));
