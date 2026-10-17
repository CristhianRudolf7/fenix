import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import type { ComponentProps } from "react";
import { cn } from "./utils";

// Local, editable primitives following the shadcn composition pattern.
const buttonVariants = cva(
  "inline-flex shrink-0 items-center justify-center gap-2 whitespace-nowrap rounded-xl text-sm font-medium transition-colors focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-[var(--accent)] disabled:pointer-events-none disabled:opacity-45",
  {
    variants: {
      variant: {
        default:
          "bg-[var(--button)] text-[var(--on-button)] hover:brightness-95",
        ghost:
          "border-0 bg-transparent text-[var(--muted)] hover:bg-[var(--hover)] hover:text-[var(--fg)]",
        outline:
          "border border-[var(--line)] bg-[var(--bg)] text-[var(--fg)] hover:bg-[var(--hover)]",
      },
      size: { default: "h-10 px-4", icon: "size-9 p-2" },
    },
    defaultVariants: { variant: "default", size: "default" },
  },
);
export function Button({
  className,
  variant,
  size,
  asChild = false,
  ...props
}: ComponentProps<"button"> &
  VariantProps<typeof buttonVariants> & { asChild?: boolean }) {
  const Component = asChild ? Slot : "button";
  return (
    <Component
      className={cn(buttonVariants({ variant, size }), className)}
      {...props}
    />
  );
}
