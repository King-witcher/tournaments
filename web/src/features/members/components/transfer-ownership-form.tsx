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
import { Select } from "@/shared/ui/select";

type Props = {
  slug: string;
  /** Membros que podem receber a posse (todos menos o owner). */
  candidates: { userId: number; nickname: string }[];
};

/** Escolher o novo owner e digitar o endereço da comunidade para confirmar. */
export function TransferOwnershipForm({ slug, candidates }: Props) {
  const t = useTranslations("Tenant.transfer");
  const router = useRouter();
  const { run, pending, error } = useApiAction();
  const [userId, setUserId] = useState("");
  const [typed, setTyped] = useState("");
  const ready = userId !== "" && typed.trim() === slug;

  return (
    <div className="flex flex-col gap-3">
      <Field label={t("member")}>
        <Select
          value={userId}
          onChange={(e) => setUserId(e.target.value)}
          className="w-64"
        >
          <option value="">{t("choose")}</option>
          {candidates.map((c) => (
            <option key={c.userId} value={c.userId}>
              {c.nickname}
            </option>
          ))}
        </Select>
      </Field>
      <Field label={t("typeSlug", { slug })}>
        <Input
          value={typed}
          onChange={(e) => setTyped(e.target.value)}
          autoComplete="off"
          className="w-64"
        />
      </Field>
      <div>
        <Button
          size="sm"
          pending={pending}
          disabled={!ready}
          onClick={() =>
            run(
              () =>
                api.POST("/tenants/{slug}/transfer-ownership", {
                  params: { path: { slug } },
                  body: { user_id: Number(userId), confirm_slug: typed.trim() },
                }),
              () => {
                setUserId("");
                setTyped("");
                router.refresh();
              },
            )
          }
        >
          {t("submit")}
        </Button>
      </div>
      {error && <ErrorText>{error}</ErrorText>}
    </div>
  );
}
