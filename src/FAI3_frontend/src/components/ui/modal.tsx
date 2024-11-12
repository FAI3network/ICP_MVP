import * as React from "react"
import { cn } from "../../utils"
import { X } from "lucide-react";
import { Button } from "./button";

interface ModalContextType {
  isOpen: boolean;
  open: () => void;
  close: () => void;
}

const ModalContext = React.createContext<ModalContextType>({
  isOpen: false,
  open: () => { },
  close: () => { },
});

let modalInstance: { open: () => void; close: () => void } | null = null;

export const Modal = React.forwardRef(({ className, ...props }: any, ref) => {
  const [isOpen, setIsOpen] = React.useState(false);

  const open = () => setIsOpen(true);
  const close = () => setIsOpen(false);

  modalInstance = { open, close };

  return (
    <ModalContext.Provider value={{ isOpen, open, close }}>
      <div className={cn("relative", className)} {...props} />
    </ModalContext.Provider>
  );
});
Modal.displayName = "Modal"

export const openModal = () => {
  if (modalInstance) {
    modalInstance.open();
  }
};

export const closeModal = () => {
  if (modalInstance) {
    modalInstance.close();
  }
};


export const ModalTrigger = React.forwardRef(({ className, ...props }: any, ref) => (
  <ModalContext.Consumer>
    {({ open }) => (
      <Button
        ref={ref}
        vairant="outline"
        className={cn(className)}
        onClick={open}
        {...props}
      />
    )}
  </ModalContext.Consumer>
))
ModalTrigger.displayName = "ModalTrigger"

export const ModalContent = React.forwardRef(({ className, closeButton = true, ...props }: any, ref) => (
  <ModalContext.Consumer>
    {({ isOpen, close }) => (
      <div
        className={cn(
          "fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50",
          className
        )}
        style={{ display: isOpen ? "flex" : "none" }}
        onClick={close}
      >
        <div
          ref={ref}
          className={cn("bg-white rounded-lg p-4 w-96 relative", className)}
          onClick={(e) => e.stopPropagation()}
          {...props}
        >
          {closeButton && (
            <button
              className="absolute top-2 right-2"
              onClick={close}
            >
              <X />
            </button>
          )}

          {props.children}
        </div>
      </div>
    )}
  </ModalContext.Consumer>

))
ModalContent.displayName = "ModalContent"

export const ModalHeader = React.forwardRef(({ className, ...props }: any, ref) => (
  <div ref={ref} className={cn("flex justify-between items-center mb-4", className)} {...props} />
))
ModalHeader.displayName = "ModalHeader"

export const ModalTitle = React.forwardRef(({ className, ...props }: any, ref) => (
  <h3 ref={ref} className={cn("text-lg font-bold", className)} {...props} />
))
ModalTitle.displayName = "ModalTitle"

export const ModalBody = React.forwardRef(({ className, ...props }: any, ref) => (
  <div ref={ref} className={cn("text-sm", className)} {...props} />
))
ModalBody.displayName = "ModalBody"

export const ModalFooter = React.forwardRef(({ className, ...props }: any, ref) => (
  <div ref={ref} className={cn("flex justify-end gap-2", className)} {...props} />
))
ModalFooter.displayName = "ModalFooter"