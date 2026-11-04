import { DELETION_CANCELLED } from "./deletion-cancelled";

/** A aba Logins, que avisa da senha nova (`PasswordResetDone`). */
const AFTER_NEW_PASSWORD = "/account/logins?password_reset=1";

/**
 * Para onde vai quem entrou por email: com a exclusão cancelada, a conta, que
 * avisa; com senha nova, a aba Logins, que avisa; senão, `returnTo`.
 */
export function afterEmailLogin(
  deletionCancelled: boolean,
  newPassword: boolean,
  returnTo: string,
) {
  if (deletionCancelled) return DELETION_CANCELLED;
  return newPassword ? AFTER_NEW_PASSWORD : returnTo;
}
