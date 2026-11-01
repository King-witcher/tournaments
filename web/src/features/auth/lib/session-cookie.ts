/**
 * O cookie que a API grava no login: o `COOKIE_NAME` dela, o mesmo daqui.
 * Lido no servidor (`server.ts`) e no proxy, que escolhe o início pela
 * presença dele.
 */
export const SESSION_COOKIE = process.env.COOKIE_NAME || "session";
