"use client";

import { useTranslations } from "next-intl";
import { type ReactNode, useEffect, useRef, useState } from "react";

import { Button } from "./button";

type Props = {
  /** Texto do botão; o mesmo vai no "sim", para dizer o que acontece. */
  children: ReactNode;
  /** O que vai acontecer, numa frase, mostrado antes do sim. */
  question: ReactNode;
  onConfirm: () => void;
  pending?: boolean;
  disabled?: boolean;
  variant?: "default" | "outline" | "link";
  size?: "default" | "sm";
  className?: string;
  /** Vermelho no sim (padrão): a ação apaga ou tira algo. `false` para as que só avançam (publicar, iniciar). */
  danger?: boolean;
};

/**
 * Botão de ação que não se desfaz fácil: o primeiro clique troca o botão pela
 * pergunta, com "sim" (o mesmo texto do botão) e "Não"; só o sim roda a
 * ação. Fica na tela, no lugar do botão, em vez do `confirm()` do navegador,
 * que não diz de onde veio e some com o contexto. Esc cancela.
 */
export function ConfirmButton({
  children,
  question,
  onConfirm,
  pending,
  disabled,
  variant = "outline",
  size = "sm",
  className,
  danger = true,
}: Props) {
  const t = useTranslations("Common");
  const [asking, setAsking] = useState(false);
  const cancelRef = useRef<HTMLButtonElement>(null);

  // O foco vai para o "Não": o botão clicado sumiu, e um Enter a mais não confirma.
  useEffect(() => {
    if (asking) cancelRef.current?.focus();
  }, [asking]);

  if (!asking) {
    return (
      <Button
        variant={variant}
        size={size}
        pending={pending}
        disabled={disabled}
        className={className}
        onClick={() => setAsking(true)}
      >
        {children}
      </Button>
    );
  }
  return (
    // min-w-0: o fieldset não encolhe abaixo do conteúdo e empurraria a página para o lado no celular.
    <fieldset
      className="min-w-0 max-w-prose"
      onKeyDown={(event) => {
        if (event.key === "Escape") setAsking(false);
      }}
    >
      <legend className="mb-2 text-sm">{question}</legend>
      <div className="flex flex-wrap gap-2">
        <Button
          variant={danger ? "destructive" : "default"}
          size="sm"
          onClick={() => {
            setAsking(false);
            onConfirm();
          }}
        >
          {children}
        </Button>
        <Button
          ref={cancelRef}
          variant="outline"
          size="sm"
          onClick={() => setAsking(false)}
        >
          {t("no")}
        </Button>
      </div>
    </fieldset>
  );
}
