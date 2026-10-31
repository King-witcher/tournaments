import { ListItem, UnorderedList } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { ConfirmModal } from "./confirm-modal";
import { TextInput } from "./text-input";

const meta = {
  title: "Ação/ConfirmModal",
  component: ConfirmModal,
  // Sai da árvore (no body) e é fixo: tela cheia na página da história, e
  // um quadro com altura própria na documentação.
  parameters: {
    layout: "fullscreen",
    docs: { story: { inline: false, height: "500px" } },
  },
  args: {
    open: true,
    onClose: fn(),
    onConfirm: fn(),
    title: "Excluir torneio",
    confirmLabel: "Excluir torneio",
    destructive: true,
    children: "O torneio e todas as séries serão apagados.",
  },
  argTypes: {
    onClose: { control: false },
    onConfirm: { control: false },
    extra: { control: false },
    details: { control: false },
    title: { control: "text" },
    confirmLabel: { control: "text" },
    children: { control: "text" },
  },
} satisfies Meta<typeof ConfirmModal>;

export default meta;
type Story = StoryObj<typeof meta>;

/** O foco começa no "Não", para um Enter a mais não confirmar. */
export const Destructive: Story = {};

/** Para o que só avança: o primário não fica vermelho. */
export const Advancing: Story = {
  args: {
    destructive: false,
    title: "Iniciar torneio",
    confirmLabel: "Iniciar torneio",
    children: "As inscrições se encerram e a chave é gerada.",
  },
};

/** Grave e raro: o primário só libera com o texto digitado, e o foco vai para o campo. */
export const ConfirmSlug: Story = {
  args: {
    slug: "copa-telefrag",
    title: "Transferir posse",
    confirmLabel: "Transferir posse",
    children: "A comunidade passa a ser da outra pessoa e você perde a posse.",
  },
};

/** Um campo a mais depois da pergunta, como a nota do histórico. */
export const WithExtra: Story = {
  args: {
    title: "Anular série",
    confirmLabel: "Anular série",
    children: "O resultado é descartado e a série volta a ser jogada.",
    extra: <TextInput labelText="Motivo" />,
  },
};

/** Com `details`, o Modal é mais largo e mostra o que vai valer antes de confirmar. */
export const WithDetails: Story = {
  args: {
    destructive: false,
    title: "Publicar torneio",
    confirmLabel: "Publicar torneio",
    children: "As inscrições abrem assim que o torneio for publicado.",
    details: (
      <UnorderedList>
        <ListItem>Formato: duelo, melhor de 3</ListItem>
        <ListItem>Vagas: 16</ListItem>
        <ListItem>Check-in: 30 minutos antes</ListItem>
      </UnorderedList>
    ),
  },
};

/** Mantém o primário travado, por exemplo enquanto falta preencher o `extra`. */
export const ConfirmDisabled: Story = {
  args: { ...WithExtra.args, confirmDisabled: true },
};
