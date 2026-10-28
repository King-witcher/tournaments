import { ButtonSet, Stack } from "@carbon/react";
import { getTranslations } from "next-intl/server";

import { ButtonLink } from "@/shared/ui/button-link";
import { getCurrentUser } from "../server";
import styles from "./anonymous-intro.module.scss";

/**
 * Na home, para quem não entrou: o que fazer para participar, com entrar
 * (primário) e criar conta à vista. Logado, não mostra nada. Fica em
 * `<Suspense>`.
 */
export async function AnonymousIntro() {
  const user = await getCurrentUser();
  if (user) return null;
  const t = await getTranslations("Home.anonymous");

  return (
    <Stack gap={6}>
      <p className={styles.hint}>{t("hint")}</p>
      {/* No ButtonSet do Carbon o primário fica à direita. */}
      <ButtonSet>
        <ButtonLink href="/signup" kind="secondary">
          {t("signup")}
        </ButtonLink>
        <ButtonLink href="/login">{t("login")}</ButtonLink>
      </ButtonSet>
    </Stack>
  );
}
