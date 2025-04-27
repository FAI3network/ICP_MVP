import { toast } from "sonner";
import { GenericError } from "../../../declarations/FAI3_backend/FAI3_backend.did";

const genericErrorToast = (err: GenericError) => {
  toast.error(
    <div className="space-y-1 bg-red-50 p-3 border-l-4 border-red-500 rounded">
      <p className="font-semibold text-red-600">Error running test</p>
      {err.category && <p className="text-red-900">Category: <span className="font-medium">{err.category}</span></p>}
      {err.code && <p className="text-red-900">Code: <span className="font-medium">{err.code.toString()}</span></p>}
      {err.details && err.details.length > 0 && (
        <div className="text-red-900">
          <p className="font-medium">Details:</p>
          <ul className="list-disc pl-4">
            {err.details.map((detail, i) => (
              <li key={i} className="text-red-800"><span className="font-medium">{detail.key}:</span> {detail.value}</li>
            ))}
          </ul>
        </div>
      )}
    </div>,
    {
      duration: 5000,
      style: { background: "#FFF3F4", color: "#B91C1C", border: "1px solid #FCA5A1" },
    }
  );
}

const errrorToast = (message: string) => {
  toast.error(
    <div className="bg-red-50 p-3 border-l-4 border-red-500 rounded">
      <p className="font-semibold text-red-600">Error</p>
      <p className="text-red-900">{message}</p>
    </div>,
    {
      duration: 5000,
      style: { background: "#FFF3F4", color: "#B91C1C", border: "1px solid #FCA5A1" },
    }
  );
}

const infoToast = (message: string) => {
  toast.info(
    <div className="bg-blue-50 p-3 border-l-4 border-blue-500 rounded">
      <p className="font-semibold text-blue-600">Info</p>
      <p className="text-blue-900">{message}</p>
    </div>,
    {
      duration: 5000,
      style: { background: "#EFF6FF", color: "#1E3A8A", border: "1px solid #BFDBFE" },
    }
  );
}

const successToast = (message: string) => {
  toast.success(
    <div className="bg-green-50 p-3 border-l-4 border-green-500 rounded">
      <p className="font-semibold text-green-600">Success</p>
      <p className="text-green-900">{message}</p>
    </div>,
    {
      duration: 5000,
      style: { background: "#ECFDF5", color: "#065F46", border: "1px solid #6EE7B7" },
    }
  );
}

const warningToast = (message: string) => {
  toast.warning(
    <div className="bg-yellow-50 p-3 border-l-4 border-yellow-500 rounded">
      <p className="font-semibold text-yellow-600">Warning</p>
      <p className="text-yellow-900">{message}</p>
    </div>,
    {
      duration: 5000,
      style: { background: "#FEFCE8", color: "#78350F", border: "1px solid #FBBF24" },
    }
  );
}

export const toasts = {
  genericErrorToast,
  errrorToast,
  infoToast,
  successToast,
  warningToast,
}