import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { ActionableNotification } from "./actionable-notification";
import { Stack } from "./carbon/stack";

const kinds = [
  "error",
  "warning",
  "warning-alt",
  "info",
  "info-square",
  "success",
] as const;

const meta = {
  title: "Estado e conteúdo/ActionableNotification",
  component: ActionableNotification,
  // Recebe o foco ao aparecer: cada história num quadro próprio, para a
  // documentação não pular até a última.
  parameters: { docs: { story: { inline: false, height: "200px" } } },
  args: {
    kind: "warning",
    title: "Check-in aberto",
    subtitle: "Confirme a presença até as 20h para entrar na chave.",
    actionButtonLabel: "Fazer check-in",
    onActionButtonClick: fn(),
    onClose: fn(),
  },
  argTypes: {
    kind: { control: "select", options: kinds },
    title: { control: "text" },
    subtitle: { control: "text" },
    caption: { control: "text" },
    actionButtonLabel: { control: "text" },
    onActionButtonClick: { control: false },
    onClose: { control: false },
    onCloseButtonClick: { control: false },
  },
} satisfies Meta<typeof ActionableNotification>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Formato de toast (o padrão): a posição fixa fica com quem usa. */
export const Toast: Story = {};

/** No fluxo da página, junto do que trata. */
export const Inline: Story = { args: { inline: true } };

export const LowContrast: Story = { args: { inline: true, lowContrast: true } };

/** `caption`: um dado curto embaixo, como a hora (só no formato de toast). */
export const WithCaption: Story = {
  args: {
    kind: "info",
    title: "Série pronta",
    subtitle: "Os dois lados fizeram o check-in.",
    caption: "21:04",
    actionButtonLabel: "Ir para a série",
  },
};

/** Sem o X: só sai pela ação. */
export const HideCloseButton: Story = {
  args: { inline: true, hideCloseButton: true },
};

export const Failure: Story = {
  args: {
    kind: "error",
    inline: true,
    title: "Partida não enviada",
    subtitle: "A conexão caiu antes do fim do envio.",
    actionButtonLabel: "Tentar de novo",
  },
};

/** Todos os tipos, em alto e em baixo contraste. */
export const Kinds: Story = {
  parameters: { docs: { story: { inline: false, height: "900px" } } },
  render: (args) => (
    <Stack gap={5}>
      {[false, true].map((lowContrast) =>
        kinds.map((kind) => (
          <ActionableNotification
            key={`${kind}-${lowContrast}`}
            {...args}
            inline
            lowContrast={lowContrast}
            kind={kind}
            title={kind}
          />
        )),
      )}
    </Stack>
  ),
};

export const Phone: Story = {
  globals: { viewport: { value: "phone", isRotated: false } },
};
