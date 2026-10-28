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
import { type MatchTarget, matchParams } from "../lib/match-target";

type Props = { target: MatchTarget; names: { a: string; b: string } };

/** Para a organização: W.O. para um dos lados, com nota opcional. */
export function WalkoverForm({ target, names }: Props) {
  const t = useTranslations("Match.organizer");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const [comment, setComment] = useState("");

  const give = (winner: "a" | "b") => {
    if (!window.confirm(t("walkoverConfirm", { name: names[winner] }))) return;
    run(
      () =>
        api.PUT(
          "/tenants/{slug}/tournaments/{tournament}/matches/{match}/result",
          {
            params: matchParams(target),
            body: {
              result: { kind: "walkover", winner },
              comment: comment.trim() || null,
            },
          },
        ),
      () => router.refresh(),
    );
  };

  return (
    <div className="flex flex-col gap-2">
      <Field label={t("comment")}>
        <Input
          value={comment}
          onChange={(e) => setComment(e.target.value)}
          maxLength={280}
        />
      </Field>
      <div className="flex flex-wrap gap-2">
        {(["a", "b"] as const).map((side) => (
          <Button
            key={side}
            size="sm"
            variant="outline"
            disabled={pending}
            onClick={() => give(side)}
          >
            {t("walkover", { name: names[side] })}
          </Button>
        ))}
      </div>
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
