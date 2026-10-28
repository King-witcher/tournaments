"use client";

import { Form, FormGroup, Stack } from "@carbon/react";
import { useTranslations } from "next-intl";
import { useActionState, useEffect, useRef, useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { callApi } from "@/shared/api/call";
import { api } from "@/shared/api/client";
import type { components } from "@/shared/api/schema";
import { useApiErrorMessage } from "@/shared/api/use-api-error-message";
import { submitAction } from "@/shared/lib/submit-action";
import { FormActions } from "@/shared/ui/form-actions";
import { FormError } from "@/shared/ui/form-error";
import { NumberInput } from "@/shared/ui/number-input";
import { RadioGroup } from "@/shared/ui/radio-group";
import { TextInput } from "@/shared/ui/text-input";
import {
  EMPTY_GAME,
  type GameValues,
  gameInput,
  gameParams,
  type MatchTarget,
} from "../lib/match-target";
import styles from "./game-form.module.scss";
import { MapSelect } from "./map-select";

type Props = {
  target: MatchTarget;
  /** Número do mapa na série. */
  number: number;
  /** `report`: jogador envia (fica pendente); `organizer`: lança, resolve ou corrige, já confirmado. */
  mode: "report" | "organizer";
  names: { a: string; b: string };
  maps: components["schemas"]["Map"][];
  /** Valores para começar (o mapa enviado, ao resolver ou corrigir). */
  initial?: GameValues;
  submitLabel: string;
  /** Depois de salvar (fecha o formulário de quem abriu). */
  onDone?: () => void;
  /** Fecha sem salvar: "Cancelar" ao lado do primário. */
  onCancel?: () => void;
  /** Abriu por um botão que sumiu: o foco vai para o primeiro campo. */
  focusOnMount?: boolean;
};

/** O que falta preencher (junto do campo) ou o erro da API (acima dos botões). */
type Errors = { winner?: string; map?: string; form?: string } | undefined;

/** Frags vão de 0 a 999 (a API confere de novo). */
const MAX_FRAGS = 999;

/** Um mapa da série: quem venceu, o mapa jogado e os frags opcionais. */
export function GameForm(props: Props) {
  const {
    target,
    number,
    mode,
    names,
    maps,
    initial,
    submitLabel,
    onDone,
    onCancel,
    focusOnMount,
  } = props;
  const t = useTranslations("Match.form");
  const errorMessage = useApiErrorMessage();
  const router = useRouter();
  const form = useRef<HTMLFormElement>(null);
  const [game, setGame] = useState<GameValues>(initial ?? EMPTY_GAME);
  const [comment, setComment] = useState("");
  // Mapa antigo, sem nome: o organizador pode manter assim.
  const mapOptional = mode === "organizer" && initial != null && !initial.map;

  useEffect(() => {
    if (focusOnMount) form.current?.querySelector("input")?.focus();
  }, [focusOnMount]);

  const [errors, submit, pending] = useActionState<Errors>(async () => {
    const { winner } = game;
    const missing = {
      winner: winner ? undefined : t("chooseWinner"),
      map: game.map || mapOptional ? undefined : t("chooseTheMap"),
    };
    if (!winner || missing.map) return missing;
    const body = gameInput({ ...game, winner });
    const params = gameParams(target, number);
    const result =
      mode === "report"
        ? await callApi(
            api.POST(
              "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/report",
              { params, body },
            ),
          )
        : await callApi(
            api.PUT(
              "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}",
              {
                params,
                body: { game: body, comment: comment.trim() || null },
              },
            ),
          );
    if (!result.ok) return { form: errorMessage(result.code, result.params) };
    setGame(EMPTY_GAME);
    onDone?.();
    router.refresh();
    return undefined;
  }, undefined);

  const frags = (side: "a" | "b") => (
    <NumberInput
      labelText={t("fragsOf", { name: names[side] })}
      value={game[side]}
      onChange={(value) => setGame({ ...game, [side]: value })}
      allowEmpty
      hideSteppers
      min={0}
      max={MAX_FRAGS}
      inputMode="numeric"
      invalidText={t("fragsRange", { max: MAX_FRAGS })}
    />
  );

  return (
    <Form
      ref={form}
      onSubmit={submitAction(submit)}
      className={styles.form}
      noValidate
    >
      <Stack gap={6}>
        {/* Os dois nomes lado a lado: um toque escolhe quem venceu. */}
        <RadioGroup
          legendText={t("winner", { number })}
          orientation="horizontal"
          options={[
            { value: "a", label: names.a },
            { value: "b", label: names.b },
          ]}
          value={game.winner}
          onChange={(winner) => setGame({ ...game, winner })}
          invalid={errors?.winner !== undefined}
          invalidText={errors?.winner}
        />
        <MapSelect
          maps={maps}
          value={game.map}
          onChange={(map) => setGame({ ...game, map })}
          invalidText={errors?.map}
        />
        <FormGroup legendText={t("frags")}>
          <div className={styles.frags}>
            {frags("a")}
            {frags("b")}
          </div>
        </FormGroup>
        {mode === "organizer" && (
          <TextInput
            labelText={t("comment")}
            value={comment}
            onChange={(e) => setComment(e.target.value)}
            maxLength={280}
          />
        )}
        {errors?.form && <FormError>{errors.form}</FormError>}
        <FormActions
          submitLabel={submitLabel}
          pending={pending}
          onCancel={onCancel}
        />
      </Stack>
    </Form>
  );
}
