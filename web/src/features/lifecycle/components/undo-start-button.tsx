"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";
import { Field } from "@/shared/ui/field";
import { Input } from "@/shared/ui/input";

type Props = {
  tenant: string;
  slug: string;
  /** Resultados que desfazer apaga. */
  results: number;
  /** Para onde volta: o texto da confirmação. */
  message: string;
};

/**
 * Desfazer o início com confirmação forte: mostra quantos resultados serão
 * apagados e, havendo algum, só libera depois de digitar o endereço do
 * torneio.
 */
export function UndoStartButton({ tenant, slug, results, message }: Props) {
  const t = useTranslations("Lifecycle");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const [open, setOpen] = useState(false);
  const [typed, setTyped] = useState("");

  if (!open) {
    return (
      <Button size="sm" variant="outline" onClick={() => setOpen(true)}>
        {t("undoStart")}
      </Button>
    );
  }
  const ready = results === 0 || typed.trim() === slug;
  return (
    <div className="flex flex-col gap-2 rounded-md border border-red-500/40 bg-red-500/5 p-3 text-sm">
      <p>{message}</p>
      <p className="font-medium">{t("undoStartResults", { count: results })}</p>
      {results > 0 && (
        <Field label={t("undoStartTypeSlug", { slug })}>
          <Input
            value={typed}
            onChange={(e) => setTyped(e.target.value)}
            autoComplete="off"
            className="w-64"
          />
        </Field>
      )}
      <div className="flex gap-2">
        <Button
          size="sm"
          pending={pending}
          disabled={!ready}
          onClick={() =>
            run(
              () =>
                api.DELETE("/tenants/{slug}/tournaments/{tournament}/start", {
                  params: { path: { slug: tenant, tournament: slug } },
                }),
              () => router.refresh(),
            )
          }
        >
          {t("undoStartSubmit")}
        </Button>
        <Button size="sm" variant="outline" onClick={() => setOpen(false)}>
          {t("cancel")}
        </Button>
      </div>
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
