"use client";

import { useTranslations } from "next-intl";
import {
  type ReactNode,
  type RefObject,
  startTransition,
  useEffect,
  useState,
} from "react";

import { usePathname } from "@/i18n/navigation";
import { formError } from "@/shared/lib/form-result";
import { FormModal } from "@/shared/ui/form-modal";
import { useCreateTenant } from "../hooks/use-create-tenant";
import { CreateTenantFields } from "./create-tenant-fields";

type Props = {
  /** O título: o nome do botão que abriu ("Criar comunidade"). */
  title: ReactNode;
  /** O botão que abriu, para o foco voltar a ele. */
  trigger?: RefObject<HTMLElement | null>;
  /** Fechou: sem criar (Cancelar, Esc, X) ou já na página da criada. */
  onClose: () => void;
};

/**
 * Criar comunidade num Modal (o painel de comunidades do cabeçalho e o
 * início): nome e endereço, e "Criar", que leva à comunidade. Um a cada
 * abertura, para começar sem valor nem erro; fechado sem criar, o foco volta
 * ao botão que abriu.
 */
export function CreateTenantModal({ title, trigger, onClose }: Props) {
  const t = useTranslations("NewTenant");
  const pathname = usePathname();
  const [openedAt] = useState(pathname);
  const form = useCreateTenant();

  // Na página da comunidade criada, o Modal sai.
  useEffect(() => {
    if (pathname !== openedAt) onClose();
  }, [pathname, openedAt, onClose]);

  return (
    <FormModal
      title={title}
      submitLabel={t("submit")}
      pending={form.pending}
      submitDisabled={!form.canSubmit}
      error={formError(form.result)}
      returnFocusTo={trigger}
      onClose={onClose}
      onSubmit={() => startTransition(form.submit)}
      formRef={form.formRef}
    >
      <CreateTenantFields form={form} />
    </FormModal>
  );
}
