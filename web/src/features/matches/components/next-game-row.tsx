import { useTranslations } from "next-intl";

import type { components } from "@/shared/api/schema";
import { Disclosure } from "@/shared/ui/disclosure";
import type { MatchTarget } from "../lib/match-target";
import { GameForm } from "./game-form";

type Schemas = components["schemas"];

type Props = {
  target: MatchTarget;
  number: number;
  names: { a: string; b: string };
  maps: Schemas["Map"][];
  /** Quem vê joga a partida: envia como jogador. */
  plays: boolean;
  /** Organizer ou owner que não joga: lança, já confirmado. */
  enters: boolean;
};

/**
 * A linha do próximo mapa: o formulário de envio para quem joga, ou
 * "aguardando envio" com o lançamento da organização fechado num botão.
 */
export function NextGameRow({
  target,
  number,
  names,
  maps,
  plays,
  enters,
}: Props) {
  const t = useTranslations("Match.games");
  const tForm = useTranslations("Match.form");

  return (
    <li className="flex flex-col gap-2 py-3">
      <span className="font-medium">{t("number", { number })}</span>
      {plays ? (
        <GameForm
          target={target}
          number={number}
          mode="report"
          names={names}
          maps={maps}
          submitLabel={tForm("submit", { number })}
        />
      ) : (
        <p className="text-sm opacity-70">{t("awaiting")}</p>
      )}
      {/* Lançar pelos jogadores é exceção: fica fechado. A chave reinicia o
          bloco a cada mapa novo. */}
      {enters && (
        <Disclosure key={number} summary={tForm("enter", { number })}>
          <GameForm
            target={target}
            number={number}
            mode="organizer"
            names={names}
            maps={maps}
            submitLabel={tForm("enter", { number })}
          />
        </Disclosure>
      )}
    </li>
  );
}
