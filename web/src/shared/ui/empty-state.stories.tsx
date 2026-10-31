import { Add } from "@carbon/icons-react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { Button } from "./button";
import { ButtonLink } from "./button-link";
import { EmptyState } from "./empty-state";

const meta = {
  title: "Estado e conteúdo/EmptyState",
  component: EmptyState,
  args: { children: "Nenhum link de inscrição." },
  argTypes: {
    children: { control: "text" },
    action: { control: false },
  },
} satisfies Meta<typeof EmptyState>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Quem vê não pode agir: só o estado. */
export const Default: Story = {};

export const WithButton: Story = {
  args: {
    action: (
      <Button renderIcon={Add} onClick={fn()}>
        Gerar link
      </Button>
    ),
  },
};

/** A ação que leva a outra página é um `ButtonLink`. */
export const WithButtonLink: Story = {
  args: {
    children: "Nenhum torneio.",
    action: <ButtonLink href="/new">Criar torneio</ButtonLink>,
  },
};

export const Phone: Story = {
  ...WithButton,
  globals: { viewport: { value: "phone", isRotated: false } },
};
