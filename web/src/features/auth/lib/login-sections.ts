/**
 * Ids dos blocos da aba Logins da conta: depois de ligar ou remover um login
 * (ou o email), o formulário some no refresh e o foco vai para o bloco
 * (`focusIfLost`).
 */
export const LOGIN_SECTION = {
  email: "login-email",
  discord: "login-discord",
  google: "login-google",
  password: "login-password",
} as const;
