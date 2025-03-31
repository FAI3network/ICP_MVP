import ClassifierTable from "./ClassifierTable";
import { useEffect, useState } from "react";
import { Button, openModal } from "../../components/ui";
import { useAuthClient, useDataContext } from "../../utils";
import { AddModelModal } from "../../components";
import LLMTable from "./LLMTable";


export default function Leaderboard() {
  const { isAdmin } = useAuthClient();
  const { Models, fetchModels, LLMModels } = useDataContext();
  const [loading, setLoading] = useState(Models.length === 0);

  useEffect(() => {
    if (Models.length > 0) return;

    (async () => {
      setLoading(true);
      await fetchModels();
      setLoading(false);
    })();
  }, [])

  useEffect(() => {
    console.log(LLMModels);
  }, [LLMModels])

  return (
    <div className="mx-20">
      <div className="flex flex-col items-center justify-center mb-4">
        <h1 className="text-4xl font-bold text-center">
          Machine Learning Model Leaderboard
        </h1>

        <p className="mt-4 text-lg text-center text-gray-500">
          Compare the performance of different machine learning models.
        </p>
      </div>
      {loading ? (
        <div className="w-full text-center">Loading...</div>
      ) : (
        <div className="w-full">
          {Models && (
            <>
              <AddModelModal />

              <div className="flex items-center justify-end py-4 mb-4 gap-3">
                {
                  isAdmin && (
                    <Button onClick={openModal}>
                      Add Model
                    </Button>
                  )
                }
              </div>
                <div className="flex flex-col xl:flex-row gap-4">
                <div className="w-full xl:w-1/2">
                  <h2 className="text-xl font-bold text-left mb-2">Classifier Models</h2>
                  <ClassifierTable />
                </div>
                <div className="w-full xl:w-1/2">
                  <h2 className="text-xl font-bold text-left mb-2">LLM Models</h2>
                  <LLMTable />
                </div>
                </div>
            </>
          )}
        </div>
      )
      }
    </div >
  );
}
