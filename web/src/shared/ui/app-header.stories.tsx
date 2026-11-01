import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { AppHeader } from "./app-header";
import { HeaderPlaceholder } from "./header-placeholder";
import { UserPanel } from "./user-panel";

const meta = {
  title: "Cabeçalho/AppHeader",
  component: AppHeader,
  parameters: {
    layout: "fullscreen",
    // A barra é fixa no topo da janela.
    docs: { story: { inline: false, height: "200px" } },
  },
  args: {
    name: "Telefrag",
    skipLabel: "Pular para o conteúdo",
    children: (
      <>
        <UserPanel
          label="Conta de Lobo"
          name="Lobo"
          items={[
            { label: "Conta", href: "/account" },
            { label: "Sair", onSelect: fn() },
          ]}
        />
      </>
    ),
  },
  argTypes: { children: { control: false } },
} satisfies Meta<typeof AppHeader>;

export default meta;
type Story = StoryObj<typeof meta>;

export const SignedIn: Story = {};

export const SignedOut: Story = {
  args: {
    children: (
      <>
        <UserPanel
          label="Entrar ou criar conta"
          items={[
            { label: "Entrar", href: "/login" },
            { label: "Criar conta", href: "/signup" },
          ]}
        />
      </>
    ),
  },
};

/** O fallback do `Suspense` de cada ação segura o lugar dela. */
export const Loading: Story = {
  args: {
    children: (
      <>
        <HeaderPlaceholder />
        <HeaderPlaceholder />
      </>
    ),
  },
};

/** Só o logo, o nome e o pular para o conteúdo. */
export const WithoutActions: Story = { args: { children: null } };

/** No celular o logo mostra só o ícone e o nome de quem está logado encolhe. */
export const Phone: Story = {
  globals: { viewport: { value: "phone", isRotated: false } },
  args: {
    children: (
      <>
        <UserPanel
          label="Conta de Lobo Solitário da Arena Brasil"
          name="Lobo Solitário da Arena Brasil"
          items={[
            { label: "Conta", href: "/account" },
            { label: "Sair", onSelect: fn() },
          ]}
        />
      </>
    ),
  },
};
