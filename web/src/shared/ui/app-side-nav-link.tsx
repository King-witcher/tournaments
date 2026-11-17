"use client";

import type { ComponentType, ReactNode } from "react";

import { type Href, Link as IntlLink, usePathname } from "@/i18n/navigation";
import { SideNavLink, SideNavMenuItem } from "./carbon/ui-shell";

type Props = {
  /** A página (sem o idioma). */
  href: Href & string;
  children: ReactNode;
  /**
   * Quando fica marcado: `exact` só no próprio endereço (o início de uma
   * área, cujo endereço é prefixo de todos os outros); `prefix` (padrão)
   * também nas páginas dentro dele.
   */
  match?: "exact" | "prefix";
  /**
   * Com `prefix`, o endereço (e o que está dentro dele) em que este não fica
   * marcado, porque outro link do menu é dele (Membros não marca o próprio
   * perfil, que é "Seu perfil").
   */
  except?: string;
  /** Dentro de um `SideNavMenu` (o segundo nível). */
  inMenu?: boolean;
};

// O link do Carbon aceita `as`, mas o tipo pede um componente sem props obrigatórias.
const NavLink = IntlLink as unknown as ComponentType;

/**
 * Um link do `AppSideNav`, marcado (`aria-current="page"`) quando o endereço
 * é o dele.
 */
export function AppSideNavLink({
  href,
  children,
  match = "prefix",
  except,
  inMenu = false,
}: Props) {
  const pathname = usePathname();
  const excepted =
    except !== undefined &&
    (pathname === except || pathname.startsWith(`${except}/`));
  const active =
    pathname === href ||
    (match === "prefix" && !excepted && pathname.startsWith(`${href}/`));
  const props = {
    as: NavLink,
    href,
    isActive: active,
    "aria-current": active ? ("page" as const) : undefined,
  };
  return inMenu ? (
    <SideNavMenuItem {...props}>{children}</SideNavMenuItem>
  ) : (
    <SideNavLink {...props}>{children}</SideNavLink>
  );
}
