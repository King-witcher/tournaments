"use client";

import { useTranslations } from "next-intl";

import { useCopyToClipboard } from "@/shared/hooks/use-copy-to-clipboard";
import { Button } from "@/shared/ui/button";
import { Input } from "@/shared/ui/input";

/** Link recém-gerado, com botão de copiar. */
export function InviteLink({ link }: { link: string }) {
  const t = useTranslations("Tenant.invites");
  const { copied, copy } = useCopyToClipboard();

  return (
    <div className="flex w-full flex-col gap-1 rounded-md border border-black/15 p-3 dark:border-white/20">
      <span className="flex gap-2">
        <Input
          readOnly
          value={link}
          aria-label={t("createdHint")}
          onFocus={(e) => e.target.select()}
          className="flex-1 font-mono text-sm"
        />
        <Button variant="outline" size="sm" onClick={() => copy(link)}>
          {copied ? t("copied") : t("copy")}
        </Button>
      </span>
      <span className="text-xs text-muted">{t("createdHint")}</span>
    </div>
  );
}
