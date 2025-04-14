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

export const toasts = {
  genericErrorToast
}