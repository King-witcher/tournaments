import { useTranslations } from "next-intl";

import { useIsClient } from "@/shared/hooks/use-is-client";
import { browserTimeZone } from "@/shared/lib/datetime-local";
import { DateTimeInput } from "@/shared/ui/date-time-input";

type Props = {
  /** Início em ISO (UTC), ou `null` sem data. */
  value: string | null;
  onChange: (startsAt: string | null) => void;
  error?: string;
  /** Na criação, a dica diz que "Criar" exige a data e o rascunho não. */
  isNew: boolean;
  /** Publicado: a data não pode ficar vazia. */
  published: boolean;
  /** O torneio já começou: não muda mais. */
  disabled: boolean;
};

/** Data e hora de início, no fuso de quem preenche. */
export function TournamentStartField({
  value,
  onChange,
  error,
  isNew,
  published,
  disabled,
}: Props) {
  const t = useTranslations("TournamentForm");
  const isClient = useIsClient();
  const timeZone = isClient ? browserTimeZone() : "…";

  return (
    <DateTimeInput
      labelText={t("startsAt")}
      helperText={t(isNew ? "startsAtHintNew" : "startsAtHint", { timeZone })}
      name="startsAt"
      value={value}
      required={published}
      disabled={disabled}
      invalid={error !== undefined}
      invalidText={error}
      onChange={onChange}
    />
  );
}
