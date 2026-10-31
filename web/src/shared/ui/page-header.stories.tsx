import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { AppContent } from "./app-content";
import { Button } from "./button";
import { OverflowActions } from "./overflow-actions";
import { Page } from "./page";
import { PageHeader } from "./page-header";
import { StatusTag } from "./status-tag";

const breadcrumbs = [
  { href: "/", label: "Início" },
  { href: "/arena-brasil", label: "Arena Brasil" },
];

const actions = (
  <>
    <Button onClick={fn()}>Inscrever-se</Button>
    <Button kind="secondary" onClick={fn()}>
      Ver as regras
    </Button>
  </>
);

const overflow = (
  <OverflowActions
    items={[
      { label: "Desfazer início", onSelect: fn() },
      {
        label: "Excluir rascunho",
        destructive: true,
        onSelect: fn(),
        confirm: { question: "O torneio e as inscrições serão apagados." },
      },
    ]}
  />
);

const meta = {
  title: "Página/PageHeader",
  component: PageHeader,
  parameters: { layout: "fullscreen" },
  decorators: [
    (Story) => (
      <AppContent>
        <Page width="wide">
          <Story />
        </Page>
      </AppContent>
    ),
  ],
  args: { title: "Copa de Duelo" },
  argTypes: {
    title: { control: "text" },
    meta: { control: "text" },
    tags: { control: false },
    actions: { control: false },
    overflow: { control: false },
    children: { control: false },
  },
} satisfies Meta<typeof PageHeader>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** A trilha vai da raiz até a página mãe; a atual é o título. */
export const WithBreadcrumbs: Story = { args: { breadcrumbs } };

export const WithTags: Story = {
  args: {
    tags: (
      <>
        <StatusTag status="registration">Inscrições abertas</StatusTag>
        <StatusTag status="neutral">Duelo</StatusTag>
      </>
    ),
  },
};

export const WithMeta: Story = {
  args: {
    meta: "Duelo · melhor de 3 · começa amanhã às 21h · 18/32 inscritos",
  },
};

export const WithActions: Story = { args: { actions } };

export const WithOverflow: Story = { args: { actions, overflow } };

export const WithDescription: Story = {
  args: {
    children: <p>Torneio aberto a toda a comunidade, com chave simples.</p>,
  },
};

export const Complete: Story = {
  args: {
    breadcrumbs,
    tags: <StatusTag status="registration">Inscrições abertas</StatusTag>,
    meta: "Duelo · melhor de 3 · começa amanhã às 21h · 18/32 inscritos",
    actions,
    overflow,
  },
};

/** O título quebra em qualquer ponto, mesmo sem espaços, e as tags descem junto. */
export const LongTitle: Story = {
  args: {
    ...Complete.args,
    title:
      "Copa de Duelo da Arena Brasil, temporada de primavera, fase final com todas as comunidades convidadas",
  },
};

/** No celular as ações descem para baixo do título. */
export const Phone: Story = {
  ...LongTitle,
  globals: { viewport: { value: "phone", isRotated: false } },
};
