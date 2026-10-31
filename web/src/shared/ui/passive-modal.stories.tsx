import { CodeSnippet } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { PassiveModal } from "./passive-modal";

const meta = {
  title: "Ação/PassiveModal",
  component: PassiveModal,
  // Sai da árvore (no body) e é fixo: tela cheia na página da história, e
  // um quadro com altura própria na documentação.
  parameters: {
    layout: "fullscreen",
    docs: { story: { inline: false, height: "500px" } },
  },
  args: {
    title: "Link de inscrição",
    onClose: fn(),
    children: (
      <CodeSnippet type="inline" feedback="Copiado">
        https://telefrag.gg/convite/k3x9v2
      </CodeSnippet>
    ),
  },
  argTypes: {
    onClose: { control: false },
    children: { control: false },
    returnFocusTo: { control: false },
    title: { control: "text" },
  },
} satisfies Meta<typeof PassiveModal>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Só informação, sem botões: fecha pelo X, Esc ou clique fora. */
export const Default: Story = {};

export const LongContent: Story = {
  args: {
    title: "Regras do torneio",
    children: (
      <p>
        As séries são melhor de 3. Quem não fizer o check-in até 15 minutos
        antes do início perde a vaga, e a chave é refeita com quem estiver
        presente. Resultados contestados são decididos por quem organiza.
      </p>
    ),
  },
};
