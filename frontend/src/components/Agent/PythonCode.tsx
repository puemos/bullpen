import DOMPurify from "dompurify";
import hljs from "highlight.js/lib/core";
import python from "highlight.js/lib/languages/python";
import { useMemo } from "react";
import "highlight.js/styles/atom-one-dark.min.css"; // Global style, but acceptable for code blocks
import { CopyButton } from "@/components/ui/copy-button";
import { cn } from "@/lib/utils";

// Register language once (outside component)
if (!hljs.getLanguage("python")) {
  hljs.registerLanguage("python", python);
}

interface PythonCodeProps {
  code: string;
  className?: string;
}

export default function PythonCode({ code, className }: PythonCodeProps) {
  const html = useMemo(
    () => DOMPurify.sanitize(hljs.highlight(code, { language: "python" }).value),
    [code],
  );

  return (
    <div className={cn("relative group rounded-md overflow-hidden text-xs", className)}>
      <div className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity">
        <CopyButton text={code} />
      </div>
      <pre className="overflow-x-auto p-3 custom-scrollbar">
        <code className="font-mono leading-relaxed" dangerouslySetInnerHTML={{ __html: html }} />
      </pre>
    </div>
  );
}
