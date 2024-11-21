import { clsx } from "clsx"
import { twMerge } from "tailwind-merge"

interface ClassValue {
  [key: string]: boolean | undefined;
}

export function cn(...inputs: (string | ClassValue | undefined)[]): string {
  return twMerge(clsx(inputs))
}
