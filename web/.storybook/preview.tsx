// Os estilos globais (Carbon e temas) antes de qualquer componente, como no
// layout, para os .module.scss virem depois deles na ordem do CSS.
import "../src/app/globals.scss";

import { withThemeByDataAttribute } from "@storybook/addon-themes";
import type { Decorator, Preview } from "@storybook/nextjs-vite";
import { NextIntlClientProvider } from "next-intl";

import { loadMessages } from "../src/i18n/namespaces";
import { routing } from "../src/i18n/routing";
import { plexMono, plexSans } from "../src/shared/lib/fonts";

// O layout põe as variáveis da fonte no <html>; aqui não há layout.
document.documentElement.classList.add(plexSans.variable, plexMono.variable);

type Locale = (typeof routing.locales)[number];

// O idioma da barra de ferramentas troca os textos dos componentes (Common,
// ApiErrors...); o conteúdo de exemplo das histórias fica em português.
const withIntl: Decorator = (Story, { globals, loaded }) => {
  const locale = globals.locale as Locale;
  document.documentElement.lang = locale;
  return (
    <NextIntlClientProvider
      locale={locale}
      messages={loaded.messages}
      timeZone={Intl.DateTimeFormat().resolvedOptions().timeZone}
    >
      <Story />
    </NextIntlClientProvider>
  );
};

const preview: Preview = {
  // Uma página de documentação por componente, com todas as variações.
  tags: ["autodocs"],
  globalTypes: {
    locale: {
      description: "Idioma",
      toolbar: {
        icon: "globe",
        items: [
          { value: "pt", title: "Português" },
          { value: "es", title: "Español" },
        ],
        dynamicTitle: true,
      },
    },
  },
  initialGlobals: { locale: routing.defaultLocale },
  loaders: [
    async ({ globals }) => ({
      messages: await loadMessages(globals.locale as Locale),
    }),
  ],
  decorators: [
    withIntl,
    // Os temas do globals.scss: white sem atributo de escolha, g100 com
    // data-theme="dark" no <html>, como o ThemeSwitcher faz.
    withThemeByDataAttribute({
      themes: { white: "light", g100: "dark" },
      defaultTheme: "white",
      attributeName: "data-theme",
    }),
  ],
  parameters: {
    // O fundo é o do tema (o body do Carbon), não o do Storybook.
    backgrounds: { disable: true },
    nextjs: { appDirectory: true },
    controls: { expanded: true },
    // As larguras que importam: o celular de referência e o breakpoint md,
    // onde os controles deixam de valer 48px.
    viewport: {
      options: {
        phone: {
          name: "Celular (390px)",
          styles: { width: "390px", height: "844px" },
          type: "mobile",
        },
        md: {
          name: "md (672px)",
          styles: { width: "672px", height: "900px" },
          type: "tablet",
        },
      },
    },
  },
};

export default preview;
