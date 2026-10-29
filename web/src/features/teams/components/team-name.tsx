import type { ReactNode } from "react";

import { MemberLink } from "@/features/members/components/member-link";
import type { components } from "@/shared/api/schema";
import { useTeamName } from "../hooks/use-team-name";
import { soloMember } from "../lib/team-name";

type Props = {
  tenant: string;
  team: components["schemas"]["Team"];
  /** No meio de uma frase: sublinhado, sem a altura de toque. */
  inline?: boolean;
  size?: "sm" | "md" | "lg";
  className?: string;
  /** O que vai dentro do link no lugar do nome (ex.: o nome num `span` que corta). */
  children?: (name: string) => ReactNode;
};

/**
 * O nome de um lado da partida (`useTeamName`). No time de um jogador (o
 * duelo), é o link para a página da pessoa, como o apelido em toda lista;
 * quem saiu da comunidade (e a conta excluída) fica sem link, porque a página
 * dele não abre mais.
 */
export function TeamName({
  tenant,
  team,
  inline,
  size,
  className,
  children,
}: Props) {
  const name = useTeamName()(team);
  const content = children ? children(name) : name;
  const solo = soloMember(team);
  if (!solo || solo.left) {
    return <span className={className}>{content}</span>;
  }
  return (
    <MemberLink
      tenant={tenant}
      userId={solo.user_id}
      inline={inline}
      size={size}
      className={className}
    >
      {content}
    </MemberLink>
  );
}
