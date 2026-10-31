import { Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn, userEvent } from "storybook/test";

import { PasswordInput } from "./password-input";

const meta = {
  title: "Formulário/PasswordInput",
  component: PasswordInput,
  args: {
    id: "password-input",
    labelText: "Senha",
    onChange: fn(),
    onBlur: fn(),
  },
  argTypes: {
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    onChange: { control: false },
    onBlur: { control: false },
  },
} satisfies Meta<typeof PasswordInput>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Required: Story = {
  args: { id: "password-input-required", required: true },
};

export const WithHelperText: Story = {
  args: {
    id: "password-input-helper",
    helperText: "Deixe em branco para manter a senha atual.",
  },
};

/** Erro que veio da API (já traduzido). */
export const Invalid: Story = {
  args: {
    id: "password-input-invalid",
    defaultValue: "segredo",
    invalid: true,
    invalidText: "Senha incorreta.",
  },
};

export const Warn: Story = {
  args: {
    id: "password-input-warn",
    defaultValue: "12345678",
    warn: true,
    warnText: "Esta senha é fácil de adivinhar.",
  },
};

export const Disabled: Story = {
  args: {
    id: "password-input-disabled",
    defaultValue: "segredo",
    disabled: true,
  },
};

/** O erro só aparece depois que a pessoa sai do campo. */
export const MinLength: Story = {
  args: {
    id: "password-input-minlength",
    labelText: "Senha nova",
    required: true,
    minLength: 8,
  },
  play: async ({ canvasElement }) => {
    const input = canvasElement.querySelector("input");
    if (!input) return;
    await userEvent.type(input, "curta");
    await userEvent.tab();
  },
};

/** A senha repetida tem de ser igual à primeira. */
export const SameAs: Story = {
  render: (args) => (
    <Stack gap={6}>
      <PasswordInput
        {...args}
        id="password-input-same-first"
        labelText="Senha nova"
        defaultValue="telefrag123"
        required
      />
      <PasswordInput
        {...args}
        id="password-input-same-second"
        labelText="Repita a senha"
        required
        sameAs="telefrag123"
      />
    </Stack>
  ),
  play: async ({ canvasElement }) => {
    const input = canvasElement.querySelector<HTMLInputElement>(
      "#password-input-same-second",
    );
    if (!input) return;
    await userEvent.type(input, "telefrag12");
    await userEvent.tab();
  },
};

export const Sizes: Story = {
  render: (args) => (
    <Stack gap={6}>
      {(["sm", "md", "lg"] as const).map((size) => (
        <PasswordInput
          key={size}
          {...args}
          id={`password-input-size-${size}`}
          labelText={size}
          size={size}
        />
      ))}
    </Stack>
  ),
};
