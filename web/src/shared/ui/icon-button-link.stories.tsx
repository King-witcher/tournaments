import { ArrowRight, Launch } from "@carbon/icons-react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";

import { Stack } from "./carbon/stack";
import { CodeSnippet } from "./code-snippet";
import { IconButtonLink } from "./icon-button-link";

const kinds = ["primary", "secondary", "tertiary", "ghost"] as const;

const meta = {
  title: "Ação/IconButtonLink",
  component: IconButtonLink,
  args: {
    href: "/veto",
    label: "Criar sala de veto",
    children: <ArrowRight />,
  },
  argTypes: {
    kind: { control: "inline-radio", options: kinds },
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    href: { control: "text" },
    children: { control: false },
  },
} satisfies Meta<typeof IconButtonLink>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Passe o mouse ou foque: o `label` aparece como dica. */
export const Ghost: Story = {};

/** Ao lado do link copiável, como na sala de veto. */
export const NewTab: Story = {
  args: { newTab: true, label: "Abrir em nova guia", children: <Launch /> },
  render: (args) => (
    <Stack orientation="horizontal">
      <CodeSnippet aria-label="Espectador">
        https://telefrag.com.br/veto/abc123
      </CodeSnippet>
      <IconButtonLink {...args} />
    </Stack>
  ),
};

/** Todos os tipos, para comparar a hierarquia. */
export const Kinds: Story = {
  render: (args) => (
    <Stack orientation="horizontal" gap={3}>
      {kinds.map((kind) => (
        <IconButtonLink key={kind} {...args} kind={kind} label={kind} />
      ))}
    </Stack>
  ),
};
