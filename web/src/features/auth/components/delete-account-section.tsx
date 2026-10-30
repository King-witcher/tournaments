import { ListItem, Stack, UnorderedList } from "@carbon/react";
import { getTranslations } from "next-intl/server";

import { getMyTenants } from "@/features/tenants/server";
import { DangerZone } from "@/shared/ui/danger-zone";
import { Link } from "@/shared/ui/link";
import { Notice } from "@/shared/ui/notice";
import { Section } from "@/shared/ui/section";
import { getLogins } from "../server";
import { DeleteAccountForm } from "./delete-account-form";
import styles from "./delete-account-section.module.scss";

/**
 * A zona de perigo da aba Perfil (`DangerZone`): excluir a conta, com o prazo, como cancelar
 * e o que acontece na data. Dono de comunidade não exclui: no lugar do
 * botão, o motivo e o link para as configurações de cada comunidade dele
 * (transferir a posse ou excluir). Fica em `<Suspense>`.
 */
export async function DeleteAccountSection() {
  const [t, logins, tenants] = await Promise.all([
    getTranslations("Account.deletion"),
    getLogins(),
    getMyTenants(),
  ]);
  if (!logins?.ok || tenants === undefined) return null;
  const owned = tenants.filter((tenant) => tenant.role === "owner");

  return (
    <DangerZone>
      <Section level={3} title={t("title")} description={t("description")}>
        <Stack gap={5}>
          <p className={styles.consequences}>{t("consequences")}</p>
          {owned.length > 0 ? (
            <>
              {/* Os links ficam fora do aviso: o InlineNotification do Carbon
                não aceita controle dentro dele. */}
              <Notice kind="info" title={t("ownerTitle")}>
                {t("owner")}
              </Notice>
              <UnorderedList>
                {owned.map((tenant) => (
                  <ListItem key={tenant.slug}>
                    <Link href={`/${tenant.slug}/settings`}>
                      {t("ownerLink", { name: tenant.name })}
                    </Link>
                  </ListItem>
                ))}
              </UnorderedList>
            </>
          ) : (
            <DeleteAccountForm password={logins.logins.username != null} />
          )}
        </Stack>
      </Section>
    </DangerZone>
  );
}
