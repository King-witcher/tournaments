import { notFound } from "next/navigation";

/**
 * Qualquer endereço que nenhuma rota atende cai aqui, dentro do layout do
 * idioma: assim o "não encontrado" sai traduzido e com o cabeçalho, em vez da
 * página padrão do Next em inglês.
 */
export default function CatchAllPage() {
  notFound();
}
