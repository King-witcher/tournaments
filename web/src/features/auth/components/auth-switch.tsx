import type { Href } from "@/i18n/navigation";
import { Link } from "@/shared/ui/link";
import styles from "./auth-switch.module.scss";

type Props = {
  /** A pergunta ("Não tem conta?"). */
  text: string;
  /** O nome da outra página ("Criar conta"). */
  label: string;
  href: Href;
};

/** Embaixo do login e do cadastro: a pergunta e o link para a outra página. */
export function AuthSwitch({ text, label, href }: Props) {
  return (
    <p className={styles.text}>
      {text}{" "}
      <Link href={href} inline>
        {label}
      </Link>
    </p>
  );
}
