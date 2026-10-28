"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";
import { Field } from "@/shared/ui/field";
import { Textarea } from "@/shared/ui/textarea";
import { type MatchTarget, matchParams } from "../lib/match-target";
import { DeadlineCountdown } from "./deadline-countdown";

type Props = {
  target: MatchTarget;
  /** Apelido de quem enviou. */
  reporter: string;
  /** Fim da janela de contestação. */
  deadline: string;
};

/**
 * Para o adversário de quem enviou: confirmar, ou contestar com um
 * comentário, com o tempo que falta antes de confirmar sozinho.
 */
export function RespondPanel({ target, reporter, deadline }: Props) {
  const t = useTranslations("Match.respond");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const [contesting, setContesting] = useState(false);
  const [comment, setComment] = useState("");
  const params = matchParams(target);

  return (
    <section className="flex flex-col gap-3 rounded-md border border-blue-500/40 p-4">
      <h2 className="text-lg font-semibold">
        {t("title", { name: reporter })}
      </h2>
      <p className="text-sm">
        {t.rich("hint", { time: () => <DeadlineCountdown until={deadline} /> })}
      </p>
      {contesting ? (
        <div className="flex flex-col gap-2">
          <Field label={t("comment")} hint={t("commentHint")}>
            <Textarea
              value={comment}
              onChange={(e) => setComment(e.target.value)}
              maxLength={280}
              rows={2}
            />
          </Field>
          <div className="flex gap-2">
            <Button
              disabled={pending}
              onClick={() =>
                run(
                  () =>
                    api.POST(
                      "/tenants/{slug}/tournaments/{tournament}/matches/{match}/contest",
                      {
                        params,
                        body: { comment },
                      },
                    ),
                  () => router.refresh(),
                )
              }
            >
              {t("submitContest")}
            </Button>
            <Button variant="outline" onClick={() => setContesting(false)}>
              {t("cancel")}
            </Button>
          </div>
        </div>
      ) : (
        <div className="flex gap-2">
          <Button
            disabled={pending}
            onClick={() =>
              run(
                () =>
                  api.POST(
                    "/tenants/{slug}/tournaments/{tournament}/matches/{match}/confirm",
                    { params },
                  ),
                () => router.refresh(),
              )
            }
          >
            {t("confirm")}
          </Button>
          <Button variant="outline" onClick={() => setContesting(true)}>
            {t("contest")}
          </Button>
        </div>
      )}
      {error && <ErrorText>{error}</ErrorText>}
    </section>
  );
}
