import type { Decorator, Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { AppContent } from "./app-content";
import { AppHeader } from "./app-header";
import { Button } from "./button";
import { Stack } from "./carbon/stack";
import { Tile } from "./carbon/tile";
import { OverflowActions } from "./overflow-actions";
import { Page } from "./page";
import { PageHeader } from "./page-header";
import { PageTabs } from "./page-tabs";
import { Section } from "./section";
import { StatusTag } from "./status-tag";
import { UserPanel } from "./user-panel";

const meta = {
  title: "Página/Page",
  component: Page,
  parameters: { layout: "fullscreen" },
  args: {
    children: (
      <>
        <PageHeader
          title="Copa de Duelo"
          meta="Duelo · melhor de 3 · 18/32 inscritos"
        />
        <Section title="Regras">
          <Tile>
            Melhor de 3 até as semifinais e melhor de 5 na final. Quem perde uma
            série cai da chave.
          </Tile>
        </Section>
        <Section title="Inscritos">
          <Tile>18 jogadores inscritos de 32 vagas.</Tile>
        </Section>
      </>
    ),
  },
  argTypes: {
    width: { control: "inline-radio", options: ["narrow", "wide"] },
    children: { control: false },
  },
} satisfies Meta<typeof Page>;

export default meta;
type Story = StoryObj<typeof meta>;

/** O `Page` dentro do `AppContent`, como nas rotas. */
const inContent: Decorator = (Story) => (
  <AppContent>
    <Story />
  </AppContent>
);

/** Texto e formulário: 8 colunas centradas a partir de `lg`. */
export const Narrow: Story = { decorators: [inContent] };

/** Tabela e chave: as 16 colunas. */
export const Wide: Story = {
  decorators: [inContent],
  args: {
    width: "wide",
    children: (
      <>
        <PageHeader title="Chave" meta="Eliminatória simples · 32 jogadores" />
        <Section title="Quartas de final">
          <Stack orientation="horizontal" gap={5}>
            {["Série 1", "Série 2", "Série 3", "Série 4"].map((name) => (
              <Tile key={name}>{name}</Tile>
            ))}
          </Stack>
        </Section>
      </>
    ),
  },
};

/** Abaixo de `md` a coluna ocupa a largura toda, nos dois modos. */
export const Phone: Story = {
  decorators: [inContent],
  globals: { viewport: { value: "phone", isRotated: false } },
};

const rotas = [
  { href: "/arena-brasil/copa-de-duelo", label: "Visão geral" },
  { href: "/arena-brasil/copa-de-duelo/inscritos", label: "Inscritos" },
  { href: "/arena-brasil/copa-de-duelo/chave", label: "Chave" },
  { href: "/arena-brasil/copa-de-duelo/regras", label: "Regras" },
];

const exemplo = (
  <>
    <AppHeader name="Telefrag" skipLabel="Pular para o conteúdo">
      <UserPanel
        label="Conta de Lobo"
        name="Lobo"
        items={[
          { label: "Conta", href: "/account" },
          { label: "Sair", onSelect: fn() },
        ]}
      />
    </AppHeader>
    <AppContent>
      <Page>
        <PageHeader
          breadcrumbs={[
            { href: "/", label: "Início" },
            { href: "/arena-brasil", label: "Arena Brasil" },
          ]}
          title="Copa de Duelo"
          tags={<StatusTag status="neutral">Rascunho</StatusTag>}
          meta="Duelo · melhor de 3 · começa amanhã às 21h · 18/32 inscritos"
          actions={
            <>
              <Button onClick={fn()}>Inscrever-se</Button>
              <Button kind="secondary" onClick={fn()}>
                Editar
              </Button>
            </>
          }
          overflow={
            <OverflowActions
              items={[
                {
                  label: "Excluir rascunho",
                  destructive: true,
                  onSelect: fn(),
                },
              ]}
            />
          }
        />
        <PageTabs tabs={rotas} label="Seções do torneio">
          <Stack gap={7}>
            <Section title="Partidas que esperam você">
              <Tile>Quartas de final · série 3 contra Raven.</Tile>
            </Section>
            <Section
              title="Inscritos"
              description="As vagas fecham no check-in."
              action={
                <Button kind="tertiary" onClick={fn()}>
                  Ver os inscritos
                </Button>
              }
            >
              <Tile>18 jogadores inscritos de 32 vagas.</Tile>
            </Section>
          </Stack>
        </PageTabs>
      </Page>
    </AppContent>
  </>
);

/** A anatomia da página: casca, cabeçalho, abas e seções. */
export const Complete: Story = {
  parameters: {
    nextjs: { navigation: { pathname: "/pt/arena-brasil/copa-de-duelo" } },
    docs: { story: { inline: false, height: "600px" } },
  },
  render: () => exemplo,
};

/** No celular as ações descem para baixo do título e as abas rolam com o dedo. */
export const CompletePhone: Story = {
  ...Complete,
  globals: { viewport: { value: "phone", isRotated: false } },
};
