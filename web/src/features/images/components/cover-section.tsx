import { getTranslations } from "next-intl/server";

import { FormSection } from "@/shared/ui/form-section";
import { getImageSettings } from "../server";
import { ImageField } from "./image-field";

type Props = {
  tenant: string;
  tournament: string;
  /** A capa de agora; nula sem capa. */
  url: string | null | undefined;
};

/**
 * A capa do torneio, na aba Geral de Editar: uma seção que se salva sozinha,
 * fora do "Salvar" do formulário. Sem o bucket público na API, nada. Fica em
 * `<Suspense>`.
 */
export async function CoverSection({ tenant, tournament, url }: Props) {
  const [settings, t] = await Promise.all([
    getImageSettings(),
    getTranslations("Images.cover"),
  ]);
  if (!settings?.enabled) return null;
  return (
    <FormSection legend={t("title")} description={t("hint")}>
      <ImageField
        target={{ kind: "cover", tenant, tournament }}
        url={url}
        maxBytes={settings.max_bytes}
      />
    </FormSection>
  );
}
