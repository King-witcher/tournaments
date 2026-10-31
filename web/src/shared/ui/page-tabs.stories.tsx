import { Tile } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { AppContent } from "./app-content";
import { Page } from "./page";
import { PageTabs } from "./page-tabs";

const base = "/arena-brasil/copa-de-duelo";

const tabs = [
  { href: base, label: "Visão geral" },
  { href: `${base}/inscritos`, label: "Inscritos" },
  { href: `${base}/chave`, label: "Chave" },
  { href: `${base}/regras`, label: "Regras" },
];

const meta = {
  title: "Página/PageTabs",
  component: PageTabs,
  parameters: {
    layout: "fullscreen",
    // A aba atual sai do endereço (o do mock traz o idioma, que o
    // `usePathname` do i18n tira).
    nextjs: { navigation: { pathname: `/pt${base}` } },
  },
  decorators: [
    (Story) => (
      <AppContent>
        <Page>
          <Story />
        </Page>
      </AppContent>
    ),
  ],
  args: {
    tabs,
    label: "Seções do torneio",
    children: <Tile>Conteúdo da aba atual.</Tile>,
  },
  argTypes: { children: { control: false } },
} satisfies Meta<typeof PageTabs>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const SecondTab: Story = {
  parameters: {
    nextjs: { navigation: { pathname: `/pt${base}/inscritos` } },
  },
  args: { children: <Tile>18 jogadores inscritos de 32 vagas.</Tile> },
};

/** Numa sub-rota da aba vale a aba de prefixo mais longo, não a primeira. */
export const NestedRoute: Story = {
  parameters: {
    nextjs: { navigation: { pathname: `/pt${base}/chave/serie-3` } },
  },
  args: { children: <Tile>Quartas de final · série 3.</Tile> },
};

/** Aba que não se aplica (Regras sem regras) simplesmente não entra. */
export const FewerTabs: Story = { args: { tabs: tabs.slice(0, 3) } };

/** Sem `children`, só a barra (o conteúdo vem de outro lugar). */
export const WithoutPanel: Story = { args: { children: undefined } };

/** Não cabendo, a barra rola com o dedo, sem as setas. */
export const ManyTabsPhone: Story = {
  globals: { viewport: { value: "phone", isRotated: false } },
  args: {
    tabs: [
      ...tabs,
      { href: `${base}/mapas`, label: "Mapas" },
      { href: `${base}/ranking`, label: "Classificação" },
      { href: `${base}/organizacao`, label: "Organização" },
    ],
  },
};
