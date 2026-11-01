import { FileUploaderSkeleton, Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { fn } from "storybook/test";

import { FileUploader } from "./file-uploader";

const meta = {
  title: "Formulário/FileUploader",
  component: FileUploader,
  args: {
    labelTitle: "Capturas de tela",
    labelDescription: "PNG ou JPG, até 2 MB cada.",
    accept: [".png", ".jpg"],
    maxFileSize: 2 * 1024 * 1024,
    multiple: true,
    filenameStatus: "edit",
    onChange: fn(),
    onDelete: fn(),
    onAddFiles: fn(),
  },
  argTypes: {
    filenameStatus: {
      control: "inline-radio",
      options: ["edit", "uploading", "complete"],
    },
    buttonKind: {
      control: "select",
      options: ["primary", "secondary", "tertiary", "ghost"],
    },
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    onChange: { control: false },
    onDelete: { control: false },
    onAddFiles: { control: false },
    onClick: { control: false },
  },
} satisfies Meta<typeof FileUploader>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Cada arquivo escolhido ganha o "x" que o tira da lista. */
export const Default: Story = {};

/** Um arquivo só: escolher outro troca o anterior. */
export const Single: Story = {
  args: {
    labelTitle: "Logo da comunidade",
    labelDescription: "PNG, até 2 MB.",
    accept: [".png"],
    multiple: false,
    buttonLabel: "Escolher a imagem",
  },
};

/** Os arquivos com o indicador de envio. */
export const Uploading: Story = {
  args: { filenameStatus: "uploading" },
};

/** Os arquivos com o visto de enviados. */
export const Complete: Story = {
  args: { filenameStatus: "complete" },
};

export const SecondaryButton: Story = {
  args: { buttonKind: "secondary" },
};

export const Disabled: Story = {
  args: { disabled: true },
};

/** No `loading.tsx`, com a forma do campo. */
export const Skeleton: Story = {
  render: () => (
    <div aria-hidden>
      <FileUploaderSkeleton />
    </div>
  ),
};

/** O botão e as linhas dos arquivos em cada tamanho. */
export const Sizes: Story = {
  render: (args) => (
    <Stack gap={7}>
      {(["sm", "md", "lg"] as const).map((size) => (
        <FileUploader key={size} {...args} labelTitle={size} size={size} />
      ))}
    </Stack>
  ),
};
