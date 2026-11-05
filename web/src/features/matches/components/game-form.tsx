"use client";

import { useTranslations } from "next-intl";
import { type ReactNode, useEffect, useRef, useState } from "react";

import type { components } from "@/shared/api/schema";
import { submitAction } from "@/shared/lib/submit-action";
import { Form } from "@/shared/ui/carbon/form";
import { Stack } from "@/shared/ui/carbon/stack";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { TextInput } from "@/shared/ui/text-input";
import { useSendGame } from "../hooks/use-send-game";
import {
  EMPTY_GAME,
  type GameValues,
  type MatchTarget,
} from "../lib/match-target";
import { validScore } from "../lib/score";
import styles from "./game-form.module.scss";
import { MapSelect } from "./map-select";
import { ScoreFields } from "./score-fields";

type Props = {
  target: MatchTarget;
  /** Número da partida na série. */
  number: number;
  /** `report`: jogador envia (fica pendente); `organizer`: lança, resolve ou corrige, já confirmado; `accept`: o capitão aceita a contestação, já confirmado. */
  mode: "report" | "organizer" | "accept";
  names: { a: string; b: string };
  /** Os mapas que cabem na escolha (sem os já jogados na série). */
  maps: components["schemas"]["Map"][];
  /** Valores para começar (o mapa enviado, ao resolver ou corrigir). */
  initial?: GameValues;
  /** O mapa que o veto deu: o envio fica preso a ele, sem a escolha. */
  fixedMap?: components["schemas"]["MapRef"];
  /** Entra depois dos campos e antes do erro e do botão (a captura de tela). */
  beforeSubmit?: ReactNode;
  submitLabel: string;
  /** Depois de salvar (fecha o formulário de quem abriu). */
  onDone?: () => void;
  /** Fecha sem salvar: "Cancelar" ao lado do primário. */
  onCancel?: () => void;
  /** Abriu por um botão que sumiu: o foco vai para o primeiro campo. */
  focusOnMount?: boolean;
  /**
   * No cartão da partida da vez: os frags nas colunas dos lados do cartão,
   * que já mostra o mapa do veto (sem a frase do mapa).
   */
  card?: boolean;
};

/**
 * Uma partida da série: o mapa jogado e o placar, ambos obrigatórios. O vencedor
 * é quem fez mais frags, para o jogador e para a organização, e o envio só
 * abre com o mapa, os dois frags na faixa e sem empate. Para a partida
 * seguinte, quem usa monta o formulário de novo (`key` com o número).
 */
export function GameForm(props: Props) {
  const { target, number, mode, names, maps, initial, fixedMap } = props;
  const { beforeSubmit, submitLabel, onDone, onCancel, focusOnMount, card } =
    props;
  const t = useTranslations("Match.form");
  const form = useRef<HTMLFormElement>(null);
  const [game, setGame] = useState<GameValues>(initial ?? EMPTY_GAME);
  const [comment, setComment] = useState("");
  const { error, pending, send } = useSendGame(target, number, mode, () => {
    setGame(EMPTY_GAME);
    onDone?.();
  });
  // O mapa só vale se o formulário o oferece: o do veto, ou um da lista (o
  // escolhido pode ter saído dela, por já ter sido jogado ou ter sido desativado).
  const offered = maps.some((m) => m.slug === game.map);
  const map = fixedMap?.slug ?? (offered ? game.map : "");
  // O botão espera o estado, e não a validade do DOM: o `NumberInput` controlado
  // atualiza o campo num commit depois do render, e o `Dropdown` nem é nativo.
  const score = validScore(game.a, game.b);

  useEffect(() => {
    if (focusOnMount)
      form.current?.querySelector<HTMLElement>("input, button")?.focus();
  }, [focusOnMount]);

  return (
    <Form
      ref={form}
      className={card ? undefined : styles.form}
      onSubmit={submitAction(
        () => score && send({ game: { map, ...score }, comment }),
      )}
    >
      <Stack gap={6}>
        {fixedMap ? (
          !card && (
            <p className={styles.fixedMap}>
              {t("vetoedMap", { map: fixedMap.name })}
            </p>
          )
        ) : (
          <div className={card ? styles.form : undefined}>
            <MapSelect
              maps={maps}
              value={map}
              onChange={(slug) => setGame({ ...game, map: slug })}
            />
          </div>
        )}
        <ScoreFields
          names={names}
          a={game.a}
          b={game.b}
          onChange={(side, value) => setGame({ ...game, [side]: value })}
          aligned={card}
        />
        {mode === "organizer" && (
          <TextInput
            labelText={t("comment")}
            value={comment}
            onChange={(e) => setComment(e.target.value)}
            maxLength={280}
          />
        )}
        {beforeSubmit}
        {error && <FormError>{error}</FormError>}
        <FormActions
          submitLabel={submitLabel}
          pending={pending}
          disabled={map === "" || !score}
          onCancel={onCancel}
        />
      </Stack>
    </Form>
  );
}
