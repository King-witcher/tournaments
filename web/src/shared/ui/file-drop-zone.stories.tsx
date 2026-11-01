import { Stack } from "@carbon/react";
import type { Meta, StoryObj } from "@storybook/nextjs-vite";
import { useState } from "react";
import { fn } from "storybook/test";

import { type DropZoneFile, FileDropZone } from "./file-drop-zone";

const MAX_SIZE = 2 * 1024 * 1024;

const sample: DropZoneFile[] = [
  { id: "1", name: "partida-1-awoken.png", status: "complete" },
  { id: "2", name: "partida-2-blood-run.png", status: "uploading" },
  { id: "3", name: "partida-3-sarnath.png", status: "edit" },
  {
    id: "4",
    name: "placar-final.bmp",
    status: "edit",
    invalid: true,
    errorSubject: "Use PNG ou JPG.",
  },
];

const meta = {
  title: "Formulário/FileDropZone",
  component: FileDropZone,
  args: {
    labelTitle: "Capturas de tela",
    labelDescription: "PNG ou JPG, até 2 MB cada.",
    accept: ["image/png", "image/jpeg"],
    maxFileSize: MAX_SIZE,
    multiple: true,
    files: [],
    onAddFiles: fn(),
    onDelete: fn(),
  },
  argTypes: {
    size: { control: "inline-radio", options: ["sm", "md", "lg"] },
    files: { control: "object" },
    onAddFiles: { control: false },
    onDelete: { control: false },
    onClick: { control: false },
  },
} satisfies Meta<typeof FileDropZone>;

export default meta;
type Story = StoryObj<typeof meta>;

/**
 * Solte arquivos na área ou clique nela: os aceitos entram na lista com o
 * "x", e os recusados aparecem com o erro.
 */
export const Default: Story = {
  render: function Render(args) {
    const [files, setFiles] = useState<DropZoneFile[]>([]);
    return (
      <FileDropZone
        {...args}
        files={files}
        onAddFiles={(event, content) => {
          args.onAddFiles?.(event, content);
          setFiles((current) => [
            ...current,
            ...content.addedFiles.map((file) => ({
              id: crypto.randomUUID(),
              name: file.name,
              status: "edit" as const,
              invalid: file.invalidFileType,
              errorSubject: file.invalidFileType
                ? file.size > MAX_SIZE
                  ? "O arquivo passa de 2 MB."
                  : "Use PNG ou JPG."
                : undefined,
            })),
          ]);
        }}
        onDelete={(id) => {
          args.onDelete(id);
          setFiles((current) => current.filter((file) => file.id !== id));
        }}
      />
    );
  },
};

/** Cada arquivo no seu estado: enviado, enviando, para tirar e recusado. */
export const WithFiles: Story = {
  args: { files: sample },
};

/** Um arquivo só. */
export const Single: Story = {
  args: {
    labelTitle: "Captura do placar",
    labelDescription: "PNG ou JPG, até 2 MB.",
    multiple: false,
    files: [sample[2]],
  },
};

export const Required: Story = {
  args: { required: true },
};

export const Disabled: Story = {
  args: { disabled: true, files: [sample[0], sample[2]] },
};

/** As linhas dos arquivos em cada tamanho. */
export const Sizes: Story = {
  render: (args) => (
    <Stack gap={7}>
      {(["sm", "md", "lg"] as const).map((size) => (
        <FileDropZone
          key={size}
          {...args}
          labelTitle={size}
          files={[sample[0], sample[2]]}
          size={size}
        />
      ))}
    </Stack>
  ),
};
