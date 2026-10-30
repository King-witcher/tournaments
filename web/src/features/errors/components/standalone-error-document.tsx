import { Stack } from "@carbon/react";
import { NextIntlClientProvider } from "next-intl";

import { cn } from "@/shared/lib/cn";
import { plexMono, plexSans } from "@/shared/lib/fonts";
import { themeInitScript } from "@/shared/lib/theme";
import { AppContent } from "@/shared/ui/app-content";
import { AppHeader } from "@/shared/ui/app-header";
import commonEs from "../../../../messages/es/Common.json";
import errorsEs from "../../../../messages/es/Errors.json";
import navEs from "../../../../messages/es/Nav.json";
import notFoundEs from "../../../../messages/es/NotFound.json";
import commonPt from "../../../../messages/pt/Common.json";
import errorsPt from "../../../../messages/pt/Errors.json";
import navPt from "../../../../messages/pt/Nav.json";
import notFoundPt from "../../../../messages/pt/NotFound.json";
import { ErrorPage } from "./error-page";

type Locale = "pt" | "es";

// Só os textos que a página usa, importados direto: aqui não há o provider
// nem o `request.ts` do layout, que é justamente o que falhou (ou o que não
// existe, no 404 sem idioma).
const messages = {
  pt: { Common: commonPt, Errors: errorsPt, Nav: navPt, NotFound: notFoundPt },
  es: { Common: commonEs, Errors: errorsEs, Nav: navEs, NotFound: notFoundEs },
};

type Props = {
  /**
   * Os idiomas, na ordem em que aparecem; o primeiro vale para o `<html>` e o
   * cabeçalho. Mais de um quando não dá para saber qual a pessoa lê (404 de
   * um endereço sem prefixo de idioma).
   */
  locales: Locale[];
  /** `failed`: erro de execução (com `retry`); `notFound`: endereço inexistente. */
  kind: "failed" | "notFound";
  retry?: () => void;
  digest?: string;
};

/**
 * O documento inteiro (`<html>` e `<body>`) das páginas de erro que
 * substituem o layout do idioma: `global-error.tsx` e `not-found.tsx` da raiz.
 * Refaz a casca do site (fontes, tema, cabeçalho com o nome, conteúdo) e põe
 * o mesmo `ErrorPage` das outras, com provider e textos próprios. Quem usa
 * importa o `globals.scss`, como o layout.
 */
export function StandaloneErrorDocument({
  locales,
  kind,
  retry,
  digest,
}: Props) {
  const [main] = locales;
  const m = messages[main];
  return (
    <html
      lang={main}
      className={cn(plexSans.variable, plexMono.variable)}
      // O script do tema põe data-theme antes da hidratação.
      suppressHydrationWarning
    >
      {/* biome-ignore lint/style/noHeadElement: é o documento inteiro, como o layout do app router */}
      <head>
        <script
          // biome-ignore lint/security/noDangerouslySetInnerHtml: constante nossa, sem entrada de usuário
          dangerouslySetInnerHTML={{ __html: themeInitScript }}
        />
      </head>
      <body>
        <NextIntlClientProvider locale={main} messages={m}>
          <AppHeader name={m.Nav.brand} skipLabel={m.Nav.skipToContent}>
            {null}
          </AppHeader>
        </NextIntlClientProvider>
        <AppContent>
          <Stack gap={9}>
            {locales.map((locale) => {
              const text =
                kind === "failed"
                  ? messages[locale].Errors
                  : messages[locale].NotFound;
              return (
                <NextIntlClientProvider
                  key={locale}
                  locale={locale}
                  messages={messages[locale]}
                >
                  <div lang={locale}>
                    <ErrorPage title={text.title} retry={retry} digest={digest}>
                      {text.text}
                    </ErrorPage>
                  </div>
                </NextIntlClientProvider>
              );
            })}
          </Stack>
        </AppContent>
      </body>
    </html>
  );
}
