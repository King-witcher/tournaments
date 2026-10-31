import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn, userEvent, within } from "storybook/test";

import { AppHeader } from "./app-header";
import { UserPanel } from "./user-panel";

const signedIn = [
  { label: "Conta", href: "/account" },
  { label: "Sair", onSelect: fn() },
];

/** Abre o painel pelo único botão fechado da barra (o nome do botão vem de uma dica que só existe ao passar o mouse). */
async function openPanel(canvasElement: HTMLElement) {
  await userEvent.click(
    await within(canvasElement).findByRole("button", { expanded: false }),
  );
}

const meta = {
  title: "Cabeçalho/UserPanel",
  component: UserPanel,
  parameters: {
    layout: "fullscreen",
    // O painel é fixo, abaixo da barra.
    docs: { story: { inline: false, height: "500px" } },
  },
  decorators: [
    (Story) => (
      <AppHeader name="Telefrag" skipLabel="Pular para o conteúdo">
        <Story />
      </AppHeader>
    ),
  ],
  args: { label: "Conta de Lobo", name: "Lobo", items: signedIn },
  argTypes: {
    label: { control: "text" },
    name: { control: "text" },
    avatarSrc: { control: "text" },
    items: { control: false },
  },
} satisfies Meta<typeof UserPanel>;

export default meta;
type Story = StoryObj<typeof meta>;

export const SignedIn: Story = {};

/**
 * Painel aberto: Conta, Sair e o tema. O seletor de tema grava `data-theme` no
 * `<html>` e no `localStorage`, e disputa com o tema da barra de ferramentas.
 */
export const SignedInOpen: Story = {
  play: ({ canvasElement }) => openPanel(canvasElement),
};

/** Sem `avatarSrc`, o ícone de usuário; com ele, a imagem. */
export const WithAvatar: Story = {
  args: { avatarSrc: "/maps/awoken.webp" },
};

export const SignedOut: Story = {
  args: {
    label: "Entrar ou criar conta",
    name: undefined,
    items: [
      { label: "Entrar", href: "/login" },
      { label: "Criar conta", href: "/signup" },
    ],
  },
};

export const SignedOutOpen: Story = {
  ...SignedOut,
  play: ({ canvasElement }) => openPanel(canvasElement),
};

/** Item de ação em andamento: ignora o clique e mostra o indicador. */
export const PendingItem: Story = {
  args: {
    items: [
      { label: "Conta", href: "/account" },
      { label: "Sair", onSelect: fn(), pending: true },
    ],
  },
  play: SignedInOpen.play,
};

/** Na barra o nome termina em reticências; no painel aparece inteiro. */
export const LongName: Story = {
  args: {
    label: "Conta de Lobo Solitário",
    name: "Lobo Solitário da Arena Brasil, campeão da Copa de Duelo",
  },
  play: ({ canvasElement }) => openPanel(canvasElement),
};

export const Phone: Story = {
  ...LongName,
  globals: { viewport: { value: "phone", isRotated: false } },
};
