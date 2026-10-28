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
import { gameParams, type MatchTarget } from "../lib/match-target";

type Props = { target: MatchTarget; number: number };

/** Para o adversário de quem enviou o mapa: confirmar, ou contestar com um comentário. */
export function GameRespond({ target, number }: Props) {
  const t = useTranslations("Match.respond");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const [contesting, setContesting] = useState(false);
  const [comment, setComment] = useState("");
  const params = gameParams(target, number);

  const confirm = () =>
    run(
      () =>
        api.POST(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/confirm",
          { params },
        ),
      () => router.refresh(),
    );
  const contest = () =>
    run(
      () =>
        api.POST(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/games/{game}/contest",
          { params, body: { comment } },
        ),
      () => router.refresh(),
    );

  return (
    <div className="flex flex-col gap-2">
      {contesting ? (
        <>
          <Field label={t("comment")} hint={t("commentHint")}>
            <Textarea
              value={comment}
              onChange={(e) => setComment(e.target.value)}
              maxLength={280}
              rows={2}
            />
          </Field>
          <div className="flex flex-wrap gap-2">
            <Button pending={pending} onClick={contest}>
              {t("submitContest")}
            </Button>
            <Button variant="outline" onClick={() => setContesting(false)}>
              {t("cancel")}
            </Button>
          </div>
        </>
      ) : (
        <div className="flex flex-wrap gap-2">
          <Button pending={pending} onClick={confirm}>
            {t("confirm")}
          </Button>
          <Button
            variant="outline"
            disabled={pending}
            onClick={() => setContesting(true)}
          >
            {t("contest")}
          </Button>
        </div>
      )}
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
