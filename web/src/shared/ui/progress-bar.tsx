"use client";

import { ProgressBar as CarbonProgressBar } from "@carbon/react";
import { useTranslations } from "next-intl";
import { type ComponentProps, useLayoutEffect, useRef } from "react";

import styles from "./progress-bar.module.scss";

type Props = ComponentProps<typeof CarbonProgressBar>;

/**
 * Barra de progresso (`ProgressBar` do Carbon) de uma tarefa que demora e que
 * a pessoa acompanha. Com `value` (de 0 a `max`), mostra quanto já foi; sem
 * ele, só que está em andamento (indeterminada). `status` `finished` ou
 * `error` encerra a barra com o ícone. `size` `small` | `big`; `type`
 * `inline` põe o rótulo ao lado da barra e `indented` alinha com o conteúdo
 * de um bloco (só dentro de tile ou painel). `helperText` diz o andamento em
 * texto ("3 de 8 partidas"). Etapas de um fluxo: `ProgressSteps`; espera sem
 * medida: `Loading` ou `InlineLoading`.
 *
 * Junto do `helperText`, o Carbon avisa o leitor de tela com "Loading" ou
 * "Done", em inglês e sem prop para trocar: o componente troca esse texto
 * pelo traduzido a cada renderização.
 */
export function ProgressBar(props: Props) {
  const t = useTranslations("Common");
  const box = useRef<HTMLDivElement>(null);
  const announcement =
    props.status === "finished" ? t("progress.complete") : t("loading");

  useLayoutEffect(() => {
    const live = box.current?.querySelector("[aria-live]");
    if (live && live.textContent !== announcement) {
      live.textContent = announcement;
    }
  });

  return (
    <div ref={box} className={styles.box}>
      <CarbonProgressBar {...props} />
    </div>
  );
}
