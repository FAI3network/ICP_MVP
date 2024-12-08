import * as RadixSelect from "@radix-ui/react-select";
import { ChevronDown, Check } from "lucide-react";
import { forwardRef } from "react";
import { cn } from "../../utils";

export const Select = ({ options, selection, setSelection }: any) => (
  <RadixSelect.Root value={selection} onValueChange={(value: string) => setSelection(value)}>
    <RadixSelect.Trigger className="w-fit inline-flex items-center justify-center rounded px-4 py-2 text-sm leading-none h-9 gap-1 bg-white shadow-md hover:bg-mauve-100 focus:outline-none focus:ring-2 focus:ring-black" aria-label="Food">
      <RadixSelect.Value placeholder="Select a fruit…" />
      <RadixSelect.Icon>
        <ChevronDown />
      </RadixSelect.Icon>
    </RadixSelect.Trigger>
    <RadixSelect.Portal>
      <RadixSelect.Content className="overflow-hidden bg-white rounded-lg shadow-lg z-50">
        <RadixSelect.ScrollUpButton className="flex items-center justify-center h-6 bg-white cursor-default">
          <ChevronDown />
        </RadixSelect.ScrollUpButton>
        <RadixSelect.Viewport className="p-1">
          <RadixSelect.Group>
            {
              options.map((option: any) => (
                <SelectItem key={option} value={option}>{option}</SelectItem>
              ))
            }
          </RadixSelect.Group>
        </RadixSelect.Viewport>
        <RadixSelect.ScrollDownButton className="flex items-center justify-center h-6 bg-white cursor-default">
          <ChevronDown />
        </RadixSelect.ScrollDownButton>
      </RadixSelect.Content>
    </RadixSelect.Portal>
  </RadixSelect.Root>
)

export const SelectItem = forwardRef(
  ({ children, className, ...props }: any, forwardedRef) => {

    return (
      <RadixSelect.Item
        className={cn("text-sm leading-none rounded flex items-center h-6 px-6 py-1 relative select-none", className)}
        {...props}
        ref={forwardedRef}
      >
        <RadixSelect.ItemText>{children}</RadixSelect.ItemText>
        <RadixSelect.ItemIndicator className="absolute left-0 w-6 flex items-center justify-center">
          <Check />
        </RadixSelect.ItemIndicator>
      </RadixSelect.Item>
    );
  },
);