import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { cn } from "@/shared/lib/cn";

type Props = { detail: components["schemas"]["MatchDetail"] };

/** Detalhe por mapa do resultado: mapa e frags de cada lado, na ordem. */
export function MatchGames({ detail }: Props) {
  const t = useTranslations("Match.games");
  const { a, b, games } = detail;

  return (
    <section className="flex flex-col gap-3">
      <h2 className="text-xl font-semibold">{t("title")}</h2>
      <table className="w-full text-sm">
        <thead className="text-left text-xs opacity-60">
          <tr>
            <th className="w-8 py-1 font-normal" />
            <th className="py-1 font-normal">{t("map")}</th>
            <th className="py-1 text-right font-normal">{a?.nickname}</th>
            <th className="py-1 text-right font-normal">{b?.nickname}</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-black/10 dark:divide-white/10">
          {games.map((g) => (
            <tr key={g.number}>
              <td className="py-1.5 tabular-nums opacity-60">
                {t("number", { number: g.number })}
              </td>
              <td className="py-1.5">{g.map.name}</td>
              <td
                className={cn(
                  "py-1.5 text-right tabular-nums",
                  g.a_frags > g.b_frags && "font-semibold",
                )}
              >
                {g.a_frags}
              </td>
              <td
                className={cn(
                  "py-1.5 text-right tabular-nums",
                  g.b_frags > g.a_frags && "font-semibold",
                )}
              >
                {g.b_frags}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}
