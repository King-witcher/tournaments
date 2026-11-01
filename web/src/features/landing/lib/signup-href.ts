import type { Href } from "@/i18n/navigation";

/**
 * "Criar conta" da apresentação: depois do cadastro, a criação da
 * comunidade. Quem chega por aqui organiza; o jogador entra pelo convite.
 */
export const SIGNUP_HREF: Href = {
  pathname: "/signup",
  query: { return_to: "/new" },
};
