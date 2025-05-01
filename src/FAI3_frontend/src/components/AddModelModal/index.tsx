import { Modal, ModalContent, ModalHeader, ModalTitle, ModalBody, Input, ModalFooter, Button, closeModal, CircularProgress } from "@/components/ui";
import { useEffect, useState } from "react";
import { useAuthClient, useDataContext } from "@/utils";
import { Toggle } from "@/components/ui/toggle";
import { ModelDetails } from "../../../../declarations/FAI3_backend/FAI3_backend.did";
import ModelDetailsForm, { FormBody } from "./ModelDetailsForm";

export default function AddModelModal({ onClose = () => { }, name = null, details = null, update = false, modelId, fetchModel, is_llm, hf_url }: { onClose?: () => void, name?: string | null, details?: ModelDetails | null, update?: boolean, modelId?: number, fetchModel?: () => Promise<any>, is_llm?: boolean, hf_url?: string }) {
  return (
    <Modal onClose={onClose}>
      <ModelDetailsForm update={update} modelId={modelId} name={name} details={details} is_llm={is_llm} hf_url={hf_url} fetchModel={fetchModel} clearModelForm={() => { }} closeModal={onClose} />
    </Modal >
  )
}

export {
  ModelDetailsForm,
  FormBody
}