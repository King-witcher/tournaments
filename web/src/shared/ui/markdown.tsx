import ReactMarkdown from "react-markdown";

import styles from "./markdown.module.scss";

/**
 * Markdown escrito por usuário (descrição, regras), com a tipografia do
 * Carbon. HTML cru não é renderizado e links perigosos (`javascript:`) são
 * descartados pelo react-markdown; links abrem em outra aba.
 */
export function Markdown({ children }: { children: string }) {
  return (
    <div className={styles.markdown}>
      <ReactMarkdown
        components={{
          a: ({ node: _node, ...props }) => (
            <a {...props} target="_blank" rel="noopener noreferrer nofollow" />
          ),
        }}
      >
        {children}
      </ReactMarkdown>
    </div>
  );
}
