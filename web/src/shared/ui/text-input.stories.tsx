import { Layer, Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { expect, fn, userEvent, within } from "storybook/test";

import { slugChars } from "@/shared/lib/input-filters";
import { TextInput } from "./text-input";

const meta = {
  title: "Formulário/TextInput",
  component: TextInput,
  args: {
    id: "text-input",
    labelText: "Nome do torneio",
    onChange: fn(),
    onBlur: fn(),
  },
  argTypes: {
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    filter: { control: false },
    onChange: { control: false },
    onBlur: { control: false },
  },
} satisfies Meta<typeof TextInput>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** A dica diz o que o campo é ou um efeito que não se vê, nunca o limite. */
export const WithHelperText: Story = {
  args: {
    id: "text-input-helper",
    labelText: "Endereço da comunidade",
    helperText: "Aparece no link de inscrição.",
  },
};

/** A estrela vermelha no rótulo e "obrigatório" para o leitor de tela. */
export const Required: Story = {
  args: { id: "text-input-required", required: true },
};

/** Erro que veio da API (já traduzido): vale mais que o do campo. */
export const Invalid: Story = {
  args: {
    id: "text-input-invalid",
    labelText: "Endereço da comunidade",
    defaultValue: "arena-sul",
    invalid: true,
    invalidText: "Este endereço já está em uso.",
  },
};

export const Warn: Story = {
  args: {
    id: "text-input-warn",
    defaultValue: "Copa",
    warn: true,
    warnText: "Já existe um torneio com nome parecido.",
  },
};

export const Disabled: Story = {
  args: {
    id: "text-input-disabled",
    defaultValue: "Copa Telefrag",
    disabled: true,
  },
};

export const ReadOnly: Story = {
  args: {
    id: "text-input-readonly",
    defaultValue: "Copa Telefrag",
    readOnly: true,
  },
};

export const Inline: Story = {
  args: { id: "text-input-inline", inline: true },
};

/** Só `slugChars` entra: maiúscula vira minúscula e o resto é descartado ao digitar. */
export const WithFilter: Story = {
  args: {
    id: "text-input-filter",
    labelText: "Endereço da comunidade",
    helperText: "Aparece no link de inscrição.",
    filter: slugChars,
    autoComplete: "off",
    spellCheck: false,
  },
  play: async ({ canvasElement }) => {
    const input = within(canvasElement).getByRole("textbox");
    await userEvent.type(input, "Arena Sul_01!");
    await expect(input).toHaveValue("arenasul01");
  },
};

/** O erro só aparece depois que a pessoa sai do campo; dali, a cada tecla. */
export const MinLength: Story = {
  args: {
    id: "text-input-minlength",
    labelText: "Endereço da comunidade",
    required: true,
    minLength: 3,
    maxLength: 32,
  },
  play: async ({ canvasElement }) => {
    const input = within(canvasElement).getByRole("textbox");
    await userEvent.type(input, "ab");
    await userEvent.tab();
  },
};

export const Pattern: Story = {
  args: {
    id: "text-input-pattern",
    labelText: "Endereço da comunidade",
    required: true,
    pattern: "[a-z0-9]+(-[a-z0-9]+)*",
    patternMessage: "Use letras minúsculas, números e hífen.",
  },
  play: async ({ canvasElement }) => {
    const input = within(canvasElement).getByRole("textbox");
    await userEvent.type(input, "Arena Sul");
    await userEvent.tab();
  },
};

export const Email: Story = {
  args: {
    id: "text-input-email",
    labelText: "Email do organizador",
    type: "email",
    required: true,
  },
  play: async ({ canvasElement }) => {
    const input = within(canvasElement).getByRole("textbox");
    await userEvent.type(input, "organizador@");
    await userEvent.tab();
  },
};

/** Regra que só o chamador sabe: aparece na hora e bloqueia o envio. */
export const Problem: Story = {
  args: {
    id: "text-input-problem",
    labelText: "Nome do torneio",
    defaultValue: "Copa Telefrag",
    problem: "Já existe um torneio com este nome.",
  },
};

/** Sobre um `Layer`, o campo troca o fundo para continuar visível. */
export const InLayer: Story = {
  render: (args) => (
    <Layer>
      <TextInput {...args} />
    </Layer>
  ),
  args: { id: "text-input-layer" },
};

/** Os estados lado a lado. */
export const States: Story = {
  render: (args) => (
    <Stack gap={6}>
      <TextInput {...args} id="text-input-states-default" />
      <TextInput
        {...args}
        id="text-input-states-helper"
        helperText="Aparece na página do torneio."
      />
      <TextInput {...args} id="text-input-states-required" required />
      <TextInput
        {...args}
        id="text-input-states-invalid"
        defaultValue="Copa"
        invalid
        invalidText="Este nome já está em uso."
      />
      <TextInput
        {...args}
        id="text-input-states-warn"
        defaultValue="Copa"
        warn
        warnText="Já existe um torneio com nome parecido."
      />
      <TextInput
        {...args}
        id="text-input-states-disabled"
        defaultValue="Copa Telefrag"
        disabled
      />
      <TextInput
        {...args}
        id="text-input-states-readonly"
        defaultValue="Copa Telefrag"
        readOnly
      />
    </Stack>
  ),
};

/** Abaixo de `md` (celular), todos ficam com 48px de altura. */
export const Sizes: Story = {
  render: (args) => (
    <Stack gap={6}>
      {(["sm", "md", "lg"] as const).map((size) => (
        <TextInput
          key={size}
          {...args}
          id={`text-input-size-${size}`}
          labelText={size}
          size={size}
        />
      ))}
    </Stack>
  ),
};
