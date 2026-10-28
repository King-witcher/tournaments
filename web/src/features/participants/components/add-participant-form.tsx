"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import { useRouter } from "@/i18n/navigation";
import { api } from "@/shared/api/client";
import { useApiAction } from "@/shared/api/use-api-action";
import { ErrorText } from "@/shared/ui/alert";
import { Button } from "@/shared/ui/button";
import { Select } from "@/shared/ui/select";

type Props = {
  tenant: string;
  tournament: string;
  /** Membros do tenant que ainda não estão inscritos. */
  candidates: { userId: number; nickname: string }[];
};

/** Para organizer e owner: inscreve um membro do tenant, em torneio público ou privado. */
export function AddParticipantForm({ tenant, tournament, candidates }: Props) {
  const t = useTranslations("Participants");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const [userId, setUserId] = useState("");

  return (
    <div className="flex flex-col gap-1">
      <div className="flex flex-wrap items-center gap-2">
        <Select
          aria-label={t("member")}
          value={userId}
          onChange={(e) => setUserId(e.target.value)}
        >
          <option value="">{t("choose")}</option>
          {candidates.map((c) => (
            <option key={c.userId} value={c.userId}>
              {c.nickname}
            </option>
          ))}
        </Select>
        <Button
          variant="outline"
          pending={pending}
          disabled={!userId}
          onClick={() =>
            run(
              () =>
                api.POST(
                  "/tenants/{slug}/tournaments/{tournament}/participants",
                  {
                    params: { path: { slug: tenant, tournament } },
                    body: { user_id: Number(userId) },
                  },
                ),
              () => {
                setUserId("");
                router.refresh();
              },
            )
          }
        >
          {t("add")}
        </Button>
      </div>
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
