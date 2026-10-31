import { Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn, userEvent } from "storybook/test";

import { TextArea } from "./text-area";

const meta = {
  title: "Formulário/TextArea",
  component: TextArea,
  args: {
    id: "text-area",
    labelText: "Regras do torneio",
    onChange: fn(),
    onBlur: fn(),
  },
  argTypes: {
    rows: { control: { type: "number", min: 2, max: 12 } },
    onChange: { control: false },
    onBlur: { control: false },
  },
} satisfies Meta<typeof TextArea>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const WithHelperText: Story = {
  args: {
    id: "text-area-helper",
    labelText: "Descrição",
    helperText: "Aparece na página do torneio.",
  },
};

export const Required: Story = {
  args: { id: "text-area-required", required: true },
};

/** Erro que veio da API (já traduzido). */
export const Invalid: Story = {
  args: {
    id: "text-area-invalid",
    defaultValue: "md3 até as quartas, md5 na final.",
    invalid: true,
    invalidText: "O texto tem caracteres que não são aceitos.",
  },
};

export const Warn: Story = {
  args: {
    id: "text-area-warn",
    defaultValue: "md3 até as quartas, md5 na final.",
    warn: true,
    warnText: "A descrição não menciona o prazo de check-in.",
  },
};

export const Disabled: Story = {
  args: {
    id: "text-area-disabled",
    defaultValue: "md3 até as quartas, md5 na final.",
    disabled: true,
  },
};

export const ReadOnly: Story = {
  args: {
    id: "text-area-readonly",
    defaultValue: "md3 até as quartas, md5 na final.",
    readOnly: true,
  },
};

/** O contador fala o idioma da interface (tente `es` na barra de ferramentas). */
export const WithCounter: Story = {
  args: {
    id: "text-area-counter",
    labelText: "Descrição",
    enableCounter: true,
    maxCount: 280,
    defaultValue: "Torneio de duelo, md3 até as quartas e md5 na final.",
  },
};

/** O erro só aparece depois que a pessoa sai do campo. */
export const MinLength: Story = {
  args: {
    id: "text-area-minlength",
    labelText: "Descrição",
    required: true,
    minLength: 20,
  },
  play: async ({ canvasElement }) => {
    const area = canvasElement.querySelector("textarea");
    if (!area) return;
    await userEvent.type(area, "md3 e md5");
    await userEvent.tab();
  },
};

export const Rows: Story = {
  args: { id: "text-area-rows", rows: 3 },
};

/** Os estados lado a lado. */
export const States: Story = {
  render: (args) => (
    <Stack gap={6}>
      <TextArea {...args} id="text-area-states-default" rows={3} />
      <TextArea
        {...args}
        id="text-area-states-required"
        rows={3}
        required
        enableCounter
        maxCount={280}
      />
      <TextArea
        {...args}
        id="text-area-states-invalid"
        rows={3}
        defaultValue="md3 até as quartas."
        invalid
        invalidText="O texto tem caracteres que não são aceitos."
      />
      <TextArea
        {...args}
        id="text-area-states-disabled"
        rows={3}
        defaultValue="md3 até as quartas."
        disabled
      />
      <TextArea
        {...args}
        id="text-area-states-readonly"
        rows={3}
        defaultValue="md3 até as quartas."
        readOnly
      />
    </Stack>
  ),
};
