import { Play, TrashCan } from "@carbon/icons-react";
import { Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { expect, fn, screen, userEvent, within } from "storybook/test";

import { ConfirmButton } from "./confirm-button";

const meta = {
  title: "Ação/ConfirmButton",
  component: ConfirmButton,
  args: {
    children: "Excluir torneio",
    question: "O torneio e todas as séries serão apagados.",
    onConfirm: fn(),
  },
  argTypes: {
    kind: {
      control: "select",
      options: [
        "primary",
        "secondary",
        "tertiary",
        "ghost",
        "danger",
        "danger--tertiary",
        "danger--ghost",
      ],
    },
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    onConfirm: { control: false },
    renderIcon: { control: false },
    question: { control: "text" },
    title: { control: "text" },
  },
} satisfies Meta<typeof ConfirmButton>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Destrutivo por padrão: gatilho `danger--tertiary` e Modal com o primário vermelho. */
export const Destructive: Story = {};

export const WithIcon: Story = { args: { renderIcon: TrashCan } };

/** Para o que só avança: gatilho `tertiary` e Modal sem o vermelho. */
export const Advancing: Story = {
  args: {
    destructive: false,
    children: "Iniciar torneio",
    question: "As inscrições se encerram e a chave é gerada.",
    renderIcon: Play,
  },
};

/** Grave e raro: o primário só libera depois de digitar o texto. */
export const ConfirmSlug: Story = {
  args: {
    confirmSlug: "copa-telefrag",
    children: "Transferir posse",
    question: "A comunidade passa a ser da outra pessoa e você perde a posse.",
  },
};

export const CustomTitle: Story = {
  args: { title: "Excluir este torneio?", children: "Excluir" },
};

export const Disabled: Story = { args: { disabled: true } };

/** Enquanto a ação roda, ignora o clique e mostra o indicador. */
export const Pending: Story = { args: { pending: true } };

/** Clica no botão para mostrar o Modal de confirmação aberto. */
export const ModalOpen: Story = {
  parameters: {
    layout: "fullscreen",
    docs: { story: { inline: false, height: "500px" } },
  },
  play: async ({ canvasElement }) => {
    await userEvent.click(within(canvasElement).getByRole("button"));
    await expect(await screen.findByRole("dialog")).toBeVisible();
  },
};

/** Com `confirmSlug`, o Modal aberto pede o texto digitado. */
export const ModalOpenSlug: Story = {
  args: ConfirmSlug.args,
  parameters: ModalOpen.parameters,
  play: ModalOpen.play,
};

/** Os gatilhos de cada tipo e estado, para comparar. */
export const Triggers: Story = {
  render: (args) => (
    <Stack orientation="horizontal" gap={3}>
      <ConfirmButton {...args}>Excluir</ConfirmButton>
      <ConfirmButton {...args} destructive={false}>
        Publicar
      </ConfirmButton>
      <ConfirmButton {...args} disabled>
        Desabilitado
      </ConfirmButton>
      <ConfirmButton {...args} pending>
        Pendente
      </ConfirmButton>
    </Stack>
  ),
};
