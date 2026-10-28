import { useTranslations } from "next-intl";

import { Link } from "@/i18n/navigation";
import type { components } from "@/shared/api/schema";
import { useMatchLabels } from "../hooks/use-match-labels";

type Schemas = components["schemas"];

type Props = {
  bracket: Schemas["TournamentBracket"];
  /** Endereço das partidas, sem a chave da partida. */
  basePath: string;
  /** Partida que não entra na lista (a da página em que ela aparece). */
  exclude?: string;
};

const ACTIONABLE: Schemas["ViewerTask"][] = ["report", "respond", "resolve"];

/**
 * As partidas que esperam uma ação de quem vê (enviar, responder ou, para a
 * organização, resolver), em destaque antes da chave: numa chave grande ou no
 * celular, o cartão da partida pode estar fora da tela. Some sem pendência.
 */
export function PendingMatches({ bracket, basePath, exclude }: Props) {
  const t = useTranslations("Bracket");
  const { code } = useMatchLabels(bracket.format.bracket === "single");
  const names = new Map(bracket.entrants.map((e) => [e.user_id, e.nickname]));
  const pending = bracket.matches.filter(
    (m) => m.key !== exclude && m.task && ACTIONABLE.includes(m.task),
  );
  if (pending.length === 0) return null;

  const name = (side: Schemas["MatchSide"]) =>
    (side.user_id != null && names.get(side.user_id)) || t("tbd");
  return (
    <section className="flex flex-col gap-2 rounded-md border border-blue-500/50 p-3">
      <h2 className="font-semibold">{t("pending.title")}</h2>
      <ul className="flex flex-col gap-1">
        {pending.map((m) => (
          <li key={m.key}>
            <Link
              href={`${basePath}/${m.key}`}
              className="inline-flex min-h-11 items-center text-sm text-link underline underline-offset-4 sm:min-h-0"
            >
              {t("pending.item", {
                match: code(m.key),
                a: name(m.a),
                b: name(m.b),
                task: m.task ? t(`task.${m.task}`) : "",
              })}
            </Link>
          </li>
        ))}
      </ul>
    </section>
  );
}
