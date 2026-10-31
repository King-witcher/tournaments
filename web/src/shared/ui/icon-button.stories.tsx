import { Add, Copy, Edit, TrashCan } from "@carbon/icons-react";
import { Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { IconButton } from "./icon-button";

const kinds = ["primary", "secondary", "tertiary", "ghost"] as const;

const meta = {
  title: "Ação/IconButton",
  component: IconButton,
  args: { label: "Editar torneio", children: <Edit />, onClick: fn() },
  argTypes: {
    kind: { control: "inline-radio", options: kinds },
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    children: { control: false },
    onClick: { control: false },
  },
} satisfies Meta<typeof IconButton>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Passe o mouse ou foque: o `label` aparece como dica. */
export const Ghost: Story = {};

export const Primary: Story = {
  args: { kind: "primary", label: "Criar torneio", children: <Add /> },
};

export const Secondary: Story = { args: { kind: "secondary" } };

export const Tertiary: Story = { args: { kind: "tertiary" } };

export const Disabled: Story = { args: { disabled: true } };

/** Ignora o clique e troca o ícone pelo indicador, sem `disabled` (o foco fica). */
export const Pending: Story = {
  args: { pending: true, label: "Copiar link", children: <Copy /> },
};

/** Todos os tipos em cada estado, para comparar a hierarquia. */
export const Kinds: Story = {
  render: (args) => (
    <Stack gap={5}>
      {[{}, { disabled: true }, { pending: true }].map((state) => (
        <Stack key={JSON.stringify(state)} orientation="horizontal" gap={3}>
          {kinds.map((kind) => (
            <IconButton
              key={kind}
              {...args}
              {...state}
              kind={kind}
              label={kind}
            >
              <TrashCan />
            </IconButton>
          ))}
        </Stack>
      ))}
    </Stack>
  ),
};

/** Abaixo de `md` (celular), todos ficam com 48px de altura. */
export const Sizes: Story = {
  render: (args) => (
    <Stack orientation="horizontal" gap={3}>
      {(["sm", "md", "lg"] as const).map((size) => (
        <IconButton key={size} {...args} size={size} label={size}>
          <Edit />
        </IconButton>
      ))}
    </Stack>
  ),
};
