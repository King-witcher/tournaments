/**
 * Os textos ficam em `messages/<idioma>/<Namespace>.json`, um arquivo por
 * namespace de primeiro nível (o primeiro argumento de `useTranslations` e
 * `getTranslations`), para cada fluxo de telas mexer só nos seus.
 *
 * Namespace novo: crie o arquivo nos dois idiomas e acrescente uma linha em
 * cada lista abaixo, em ordem alfabética. Os tipos do next-intl saem de `pt`;
 * `es` precisa ter as mesmas chaves (falta de chave em `es` é erro de
 * compilação). Os imports são estáticos (nada de fs em runtime) e cada
 * arquivo vira um módulo à parte, carregado só no servidor.
 */

const pt = {
  Account: () => import("../../messages/pt/Account.json"),
  ApiErrors: () => import("../../messages/pt/ApiErrors.json"),
  Auth: () => import("../../messages/pt/Auth.json"),
  Bracket: () => import("../../messages/pt/Bracket.json"),
  Common: () => import("../../messages/pt/Common.json"),
  Discord: () => import("../../messages/pt/Discord.json"),
  Home: () => import("../../messages/pt/Home.json"),
  Invite: () => import("../../messages/pt/Invite.json"),
  Lifecycle: () => import("../../messages/pt/Lifecycle.json"),
  LocaleSwitcher: () => import("../../messages/pt/LocaleSwitcher.json"),
  Locales: () => import("../../messages/pt/Locales.json"),
  Login: () => import("../../messages/pt/Login.json"),
  Match: () => import("../../messages/pt/Match.json"),
  Member: () => import("../../messages/pt/Member.json"),
  Metadata: () => import("../../messages/pt/Metadata.json"),
  Nav: () => import("../../messages/pt/Nav.json"),
  NewTenant: () => import("../../messages/pt/NewTenant.json"),
  NewTournament: () => import("../../messages/pt/NewTournament.json"),
  NotFound: () => import("../../messages/pt/NotFound.json"),
  Notifications: () => import("../../messages/pt/Notifications.json"),
  Organizing: () => import("../../messages/pt/Organizing.json"),
  Participants: () => import("../../messages/pt/Participants.json"),
  Screenshots: () => import("../../messages/pt/Screenshots.json"),
  Ratings: () => import("../../messages/pt/Ratings.json"),
  Roles: () => import("../../messages/pt/Roles.json"),
  Signup: () => import("../../messages/pt/Signup.json"),
  Team: () => import("../../messages/pt/Team.json"),
  Tenant: () => import("../../messages/pt/Tenant.json"),
  ThemeSwitcher: () => import("../../messages/pt/ThemeSwitcher.json"),
  Tournament: () => import("../../messages/pt/Tournament.json"),
  TournamentForm: () => import("../../messages/pt/TournamentForm.json"),
  TournamentStatus: () => import("../../messages/pt/TournamentStatus.json"),
  Tournaments: () => import("../../messages/pt/Tournaments.json"),
  Visibility: () => import("../../messages/pt/Visibility.json"),
};

/** O conteúdo de cada namespace, em português (a referência dos tipos). */
export type Messages = {
  [N in keyof typeof pt]: Awaited<ReturnType<(typeof pt)[N]>>["default"];
};

type Loaders = {
  [N in keyof Messages]: () => Promise<{ default: Messages[N] }>;
};

const es: Loaders = {
  Account: () => import("../../messages/es/Account.json"),
  ApiErrors: () => import("../../messages/es/ApiErrors.json"),
  Auth: () => import("../../messages/es/Auth.json"),
  Bracket: () => import("../../messages/es/Bracket.json"),
  Common: () => import("../../messages/es/Common.json"),
  Discord: () => import("../../messages/es/Discord.json"),
  Home: () => import("../../messages/es/Home.json"),
  Invite: () => import("../../messages/es/Invite.json"),
  Lifecycle: () => import("../../messages/es/Lifecycle.json"),
  LocaleSwitcher: () => import("../../messages/es/LocaleSwitcher.json"),
  Locales: () => import("../../messages/es/Locales.json"),
  Login: () => import("../../messages/es/Login.json"),
  Match: () => import("../../messages/es/Match.json"),
  Member: () => import("../../messages/es/Member.json"),
  Metadata: () => import("../../messages/es/Metadata.json"),
  Nav: () => import("../../messages/es/Nav.json"),
  NewTenant: () => import("../../messages/es/NewTenant.json"),
  NewTournament: () => import("../../messages/es/NewTournament.json"),
  NotFound: () => import("../../messages/es/NotFound.json"),
  Notifications: () => import("../../messages/es/Notifications.json"),
  Organizing: () => import("../../messages/es/Organizing.json"),
  Participants: () => import("../../messages/es/Participants.json"),
  Screenshots: () => import("../../messages/es/Screenshots.json"),
  Ratings: () => import("../../messages/es/Ratings.json"),
  Roles: () => import("../../messages/es/Roles.json"),
  Signup: () => import("../../messages/es/Signup.json"),
  Team: () => import("../../messages/es/Team.json"),
  Tenant: () => import("../../messages/es/Tenant.json"),
  ThemeSwitcher: () => import("../../messages/es/ThemeSwitcher.json"),
  Tournament: () => import("../../messages/es/Tournament.json"),
  TournamentForm: () => import("../../messages/es/TournamentForm.json"),
  TournamentStatus: () => import("../../messages/es/TournamentStatus.json"),
  Tournaments: () => import("../../messages/es/Tournaments.json"),
  Visibility: () => import("../../messages/es/Visibility.json"),
};

const loaders: Record<"pt" | "es", Loaders> = { pt, es };

/** Todos os namespaces de `locale`, no formato que o next-intl espera. */
export async function loadMessages(locale: "pt" | "es"): Promise<Messages> {
  const entries = await Promise.all(
    Object.entries(loaders[locale]).map(
      async ([namespace, load]) => [namespace, (await load()).default] as const,
    ),
  );
  return Object.fromEntries(entries) as Messages;
}
