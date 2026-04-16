import { Copy, Check } from "@phosphor-icons/react";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { useCopyToClipboard } from "@/hooks/useCopyToClipboard";
import { cn } from "@/lib/utils";
import type { VariantProps } from "class-variance-authority";
import { buttonVariants } from "@/components/ui/button";

interface CopyButtonProps {
  text: string;
  size?: VariantProps<typeof buttonVariants>["size"];
  iconSize?: number;
  variant?: VariantProps<typeof buttonVariants>["variant"];
  className?: string;
  tooltipText?: string;
}

export function CopyButton({
  text,
  size = "icon-xs",
  iconSize = 14,
  variant = "ghost",
  className,
  tooltipText = "Copy",
}: CopyButtonProps) {
  const { copied, copy } = useCopyToClipboard();

  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant={variant}
            size={size}
            className={cn("text-muted-foreground", className)}
            onClick={(e) => {
              e.stopPropagation();
              copy(text);
            }}
          >
            {copied ? (
              <Check size={iconSize} className="text-green-500" />
            ) : (
              <Copy size={iconSize} />
            )}
          </Button>
        </TooltipTrigger>
        <TooltipContent>{copied ? "Copied!" : tooltipText}</TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}
