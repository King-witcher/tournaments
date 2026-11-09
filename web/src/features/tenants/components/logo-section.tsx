import { getTranslations } from "next-intl/server";
import { ImageField } from "@/features/images/components/image-field";
import { getImageSettings } from "@/features/images/server";
import { FormSection } from "@/shared/ui/form-section";

type Props = {
  slug: string;
  /** O logo de agora; nulo sem logo. */
  url: string | null | undefined;
};

/**
 * O logo da comunidade, em Geral: um bloco que se salva sozinho, ao lado do
 * formulário. Sem o bucket público na API, nada. Fica em `<Suspense>`.
 */
export async function LogoSection({ slug, url }: Props) {
  const [settings, t] = await Promise.all([
    getImageSettings(),
    getTranslations("Images.logo"),
  ]);
  if (!settings?.enabled) return null;

  return (
    <FormSection legend={t("title")} description={t("hint")}>
      <ImageField
        target={{ kind: "logo", tenant: slug }}
        url={url}
        maxBytes={settings.max_bytes}
      />
    </FormSection>
  );
}
