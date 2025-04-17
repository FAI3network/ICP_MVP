import * as React from "react"
import { cn } from "../../utils"
import { X } from "lucide-react";
import { Button } from "./button";

interface ModalContextType {
  isModalOpen: boolean;
  open: () => void;
  close: () => void;
}

const ModalContext = React.createContext<ModalContextType>({
  isModalOpen: false,
  open: () => { },
  close: () => { },
});

let modalInstance: { open: () => void; close: () => void } | null = null;

export const Modal = React.forwardRef(({ className, onClose = null, isOpen = false, ...props }: any, ref) => {
  const [isModalOpen, setIsModalOpen] = React.useState(false);

  const open = () => setIsModalOpen(true);
  const close = () => {
    setIsModalOpen(false)
    onClose && onClose()
  };

  React.useEffect(() => {
    if (isOpen) {
      open();
    } else {
      close();
    }
  }, [isOpen]);

  modalInstance = { open, close };

  return (
    <ModalContext.Provider value={{ isModalOpen, open, close }}>
      <div
        className={cn(
          "fixed inset-0 z-30 bg-black bg-opacity-50 p-4 w-full h-full",
          className
        )}
        style={{ display: isModalOpen ? "flex" : "none" }}
        onClick={close}
      >
        <div className={cn("relative w-full h-full flex items-center justify-center", className)} {...props} />
      </div>
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
    {({ isModalOpen, close }) => (
      <div
        ref={ref}
        className={cn("bg-white max-w-full max-h-full rounded-lg p-4 relative overflow-y-auto", className)}
        onClick={(e) => { e.stopPropagation(); e.preventDefault(); }}
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