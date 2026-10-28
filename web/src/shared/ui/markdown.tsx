import ReactMarkdown from "react-markdown";

/**
 * Markdown escrito por usuário (descrição, regras). HTML cru não é
 * renderizado e links perigosos (`javascript:`) são descartados pelo
 * react-markdown; links abrem em outra aba.
 */
export function Markdown({ children }: { children: string }) {
  return (
    <div className="flex flex-col gap-3 text-sm leading-relaxed [&_a]:underline [&_blockquote]:border-l-2 [&_blockquote]:pl-3 [&_blockquote]:text-muted [&_code]:font-mono [&_h1]:text-xl [&_h1]:font-semibold [&_h2]:text-lg [&_h2]:font-semibold [&_h3]:font-semibold [&_ol]:list-decimal [&_ol]:pl-5 [&_ul]:list-disc [&_ul]:pl-5">
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
