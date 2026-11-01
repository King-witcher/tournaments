"use client";

import { useTranslations } from "next-intl";
import { useEffect, useRef, useState, useTransition } from "react";

import { type Href, useRouter } from "@/i18n/navigation";
import { Button } from "@/shared/ui/button";
import { ButtonLink } from "@/shared/ui/button-link";
import { Notice } from "@/shared/ui/notice";
import { useImageErrorMessage } from "../hooks/use-image-error-message";
import type { UnsentImage } from "../hooks/use-image-on-create";
import { sendImage } from "../lib/upload-image";
import styles from "./image-not-sent.module.scss";

type Props = {
  unsent: UnsentImage;
  /** O que foi criado, para onde seguir. */
  href: Href;
};

/**
 * No lugar do formulário de criação, quando a comunidade ou o torneio foram
 * criados e o logo ou a capa não chegaram: o motivo, "Tentar de novo" (com a
 * mesma imagem) e seguir sem ela. O foco vai para "Tentar de novo", porque o
 * botão de enviar sumiu.
 */
export function ImageNotSent({ unsent, href }: Props) {
  const t = useTranslations("Images.notSent");
  const errorMessage = useImageErrorMessage();
  const router = useRouter();
  const retryRef = useRef<HTMLButtonElement>(null);
  const [pending, startTransition] = useTransition();
  const [failure, setFailure] = useState(unsent.failure);
  const { kind } = unsent.target;

  useEffect(() => retryRef.current?.focus(), []);

  const retry = () =>
    startTransition(async () => {
      const sent = await sendImage(unsent.target, unsent.image);
      if (sent.ok) {
        router.push(href);
        router.refresh();
      } else {
        setFailure(sent);
      }
    });

  return (
    <Notice
      kind="error"
      title={t(`${kind}.title`)}
      action={
        <div className={styles.buttons}>
          <Button ref={retryRef} pending={pending} onClick={retry}>
            {t("retry")}
          </Button>
          <ButtonLink href={href} kind="secondary">
            {t(`${kind}.skip`)}
          </ButtonLink>
        </div>
      }
    >
      {errorMessage(failure)}
    </Notice>
  );
}
