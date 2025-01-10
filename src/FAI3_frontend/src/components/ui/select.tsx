import * as RadixSelect from "@radix-ui/react-select";
import { ChevronDown, Check } from "lucide-react";
import { forwardRef } from "react";
import { cn } from "../../utils";
import { useState, useEffect } from "react";

export const Select = ({ options, selection, setSelection, multiple = false }: any) => {
  const [multipleSelection, setMultipleSelection] = useState<string[]>([]);

  const handleSelection = (value: string) => {
    if (multiple) {
      if (multipleSelection.includes(value)) {
        setMultipleSelection(multipleSelection.filter((item) => item !== value));
      } else {
        setMultipleSelection([...multipleSelection, value]);
      }
    } else {
      setSelection(value);
    }
  }

  useEffect(() => {
    if (multiple) {
      const multipleString = multipleSelection.join(", ");
      setSelection(multipleString);
      console.log(multipleString);
      console.log(multipleSelection);
    }
  }, [multipleSelection]);

  return (
    <RadixSelect.Root onValueChange={(value: string) => handleSelection(value)}>
      <RadixSelect.Trigger className="w-fit inline-flex items-center justify-center rounded px-4 py-2 text-sm leading-none h-9 gap-1 bg-white shadow-md hover:bg-mauve-100 focus:outline-none focus:ring-2 focus:ring-black" aria-label="Food">
        <RadixSelect.Value placeholder="Select a column…" >
          {selection.length > 32 ? selection.slice(0, 24) + "..." : selection}
          {selection.length == 0 && "Select a column..."}
        </RadixSelect.Value>
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
                options.map((option: any) => {
                  const isSelected = multiple ? multipleSelection.includes(option) : selection === option;

                  return (
                    <SelectItem key={option} value={option} selected={isSelected} >{option}</SelectItem>
                  )
                })
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
}

export const SelectItem = forwardRef(
  ({ children, className, selected, ...props }: any, forwardedRef) => {

    return (
      <RadixSelect.Item
        className={cn("text-sm leading-none rounded flex items-center h-6 px-6 py-1 relative select-none cursor-pointer hover:bg-slate-200", className)}
        {...props}
        ref={forwardedRef}
      >
        <RadixSelect.ItemText>{children}</RadixSelect.ItemText>
        {
          selected && (
            <div className="absolute left-0 w-6 flex items-center justify-center">
              <Check />
            </div>
          ) 
          // : (
          //   <RadixSelect.ItemIndicator className="absolute left-0 w-6 flex items-center justify-center">
          //     <Check />
          //   </RadixSelect.ItemIndicator>
          // )
        }
        {/* <div className="absolute left-0 w-6 flex items-center justify-center">
          <Check />
        </div> */}

        {/* <RadixSelect.ItemIndicator className="absolute left-0 w-6 flex items-center justify-center">
          <Check />
        </RadixSelect.ItemIndicator> */}
      </RadixSelect.Item>
    );
  },
);