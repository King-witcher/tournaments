import type { ServerMessage } from "./realtime-client";

type Moved = Extract<ServerMessage, { type: "moved" }>;

/**
 * O caminho no endereço novo, se `pathname` (sem o idioma) está na comunidade
 * ou no torneio que mudou (`moved`); `undefined` se não está. Com `tournament`,
 * `to` é o slug novo do torneio; sem, o da comunidade.
 */
export function movedPath(pathname: string, moved: Moved): string | undefined {
  const { tenant, tournament, to } = moved;
  const from = tournament ? `/${tenant}/${tournament}` : `/${tenant}`;
  if (pathname !== from && !pathname.startsWith(`${from}/`)) return undefined;
  const target = tournament ? `/${tenant}/${to}` : `/${to}`;
  return `${target}${pathname.slice(from.length)}`;
}
