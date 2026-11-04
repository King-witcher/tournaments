/**
 * Os textos ficam em `messages/<idioma>/<Namespace>.json`, um arquivo por
 * namespace de primeiro nível (o primeiro argumento de `useTranslations` e
 * `getTranslations`), para cada fluxo de telas mexer só nos seus.
 *
 * Namespace novo: crie o arquivo nos três idiomas e acrescente uma linha em
 * cada lista abaixo, em ordem alfabética. Os tipos do next-intl saem de `pt`;
 * `es` e `en` precisam ter as mesmas chaves (falta de chave é erro de
 * compilação). Os imports são estáticos (nada de fs em runtime) e cada
 * arquivo vira um módulo à parte, carregado só no servidor.
 */

const pt = {
  Account: () => import("../../messages/pt/Account.json"),
  Admission: () => import("../../messages/pt/Admission.json"),
  ApiErrors: () => import("../../messages/pt/ApiErrors.json"),
  Auth: () => import("../../messages/pt/Auth.json"),
  Bracket: () => import("../../messages/pt/Bracket.json"),
  Common: () => import("../../messages/pt/Common.json"),
  Discord: () => import("../../messages/pt/Discord.json"),
  Errors: () => import("../../messages/pt/Errors.json"),
  Home: () => import("../../messages/pt/Home.json"),
  Images: () => import("../../messages/pt/Images.json"),
  Invite: () => import("../../messages/pt/Invite.json"),
  Landing: () => import("../../messages/pt/Landing.json"),
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
  VetoRoom: () => import("../../messages/pt/VetoRoom.json"),
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
  Admission: () => import("../../messages/es/Admission.json"),
  ApiErrors: () => import("../../messages/es/ApiErrors.json"),
  Auth: () => import("../../messages/es/Auth.json"),
  Bracket: () => import("../../messages/es/Bracket.json"),
  Common: () => import("../../messages/es/Common.json"),
  Discord: () => import("../../messages/es/Discord.json"),
  Errors: () => import("../../messages/es/Errors.json"),
  Home: () => import("../../messages/es/Home.json"),
  Images: () => import("../../messages/es/Images.json"),
  Invite: () => import("../../messages/es/Invite.json"),
  Landing: () => import("../../messages/es/Landing.json"),
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
  VetoRoom: () => import("../../messages/es/VetoRoom.json"),
};

const en: Loaders = {
  Account: () => import("../../messages/en/Account.json"),
  Admission: () => import("../../messages/en/Admission.json"),
  ApiErrors: () => import("../../messages/en/ApiErrors.json"),
  Auth: () => import("../../messages/en/Auth.json"),
  Bracket: () => import("../../messages/en/Bracket.json"),
  Common: () => import("../../messages/en/Common.json"),
  Discord: () => import("../../messages/en/Discord.json"),
  Errors: () => import("../../messages/en/Errors.json"),
  Home: () => import("../../messages/en/Home.json"),
  Images: () => import("../../messages/en/Images.json"),
  Invite: () => import("../../messages/en/Invite.json"),
  Landing: () => import("../../messages/en/Landing.json"),
  Lifecycle: () => import("../../messages/en/Lifecycle.json"),
  LocaleSwitcher: () => import("../../messages/en/LocaleSwitcher.json"),
  Locales: () => import("../../messages/en/Locales.json"),
  Login: () => import("../../messages/en/Login.json"),
  Match: () => import("../../messages/en/Match.json"),
  Member: () => import("../../messages/en/Member.json"),
  Metadata: () => import("../../messages/en/Metadata.json"),
  Nav: () => import("../../messages/en/Nav.json"),
  NewTenant: () => import("../../messages/en/NewTenant.json"),
  NewTournament: () => import("../../messages/en/NewTournament.json"),
  NotFound: () => import("../../messages/en/NotFound.json"),
  Notifications: () => import("../../messages/en/Notifications.json"),
  Organizing: () => import("../../messages/en/Organizing.json"),
  Participants: () => import("../../messages/en/Participants.json"),
  Screenshots: () => import("../../messages/en/Screenshots.json"),
  Ratings: () => import("../../messages/en/Ratings.json"),
  Roles: () => import("../../messages/en/Roles.json"),
  Signup: () => import("../../messages/en/Signup.json"),
  Team: () => import("../../messages/en/Team.json"),
  Tenant: () => import("../../messages/en/Tenant.json"),
  ThemeSwitcher: () => import("../../messages/en/ThemeSwitcher.json"),
  Tournament: () => import("../../messages/en/Tournament.json"),
  TournamentForm: () => import("../../messages/en/TournamentForm.json"),
  TournamentStatus: () => import("../../messages/en/TournamentStatus.json"),
  Tournaments: () => import("../../messages/en/Tournaments.json"),
  VetoRoom: () => import("../../messages/en/VetoRoom.json"),
};

const loaders: Record<"pt" | "es" | "en", Loaders> = { pt, es, en };

/** Todos os namespaces de `locale`, no formato que o next-intl espera. */
export async function loadMessages(
  locale: "pt" | "es" | "en",
): Promise<Messages> {
  const entries = await Promise.all(
    Object.entries(loaders[locale]).map(
      async ([namespace, load]) => [namespace, (await load()).default] as const,
    ),
  );
  return Object.fromEntries(entries) as Messages;
}
