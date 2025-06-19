import { Link } from "react-router-dom";
import { Actor, HttpAgent } from "@dfinity/agent";
import { Job } from "../../../declarations/FAI3_backend/FAI3_backend.did";
import { useEffect, useState, useContext, useRef, useCallback } from "react";
import { Button, CircularProgress } from "./ui";
import { useAuthClient, formatAddress, useDataContext } from "../utils";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover"
import { Principal } from "@dfinity/principal";

export default function Navbar() {
  const { authClient, address, webapp, connect, disconnect, connecting } = useAuthClient();
  const { workerProcesses } = useDataContext();
  const [jobs, setJobs] = useState<{ [key: number]: Job } | null>(null);
  const [stopQueue, setStopQueue] = useState<number[]>([]);
  const [hasCheckedExistingJobs, setHasCheckedExistingJobs] = useState(false);

  const timeoutRef = useRef<number | null>(null);
  const isPollingRef = useRef(false);

  const copyAddress = async () => {
    await navigator.clipboard.writeText(address);
    const tooltip = document.getElementById("tooltip");
    if (tooltip) {
      tooltip.style.opacity = "1";
      setTimeout(() => {
        tooltip.style.opacity = "0";
      }, 1000);
    }
  }

  const clearPolling = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }
    isPollingRef.current = false;
  }, []);

  const removeJob = useCallback((jobId: number) => {
    setJobs((prevJobs: any) => {
      if (!prevJobs) return prevJobs;
      const newJobs = { ...prevJobs };
      delete newJobs[jobId];
      return newJobs;
    });
  }, []);

  const shouldContinuePolling = useCallback(() => {
    const hasActiveJobs = jobs && Object.keys(jobs).length > 0;
    const hasWorkerProcesses = workerProcesses.length > 0;
    return hasActiveJobs || hasWorkerProcesses;
  }, [jobs, workerProcesses]);


  const queryJobStatus = useCallback(async () => {
    if (!webapp || isPollingRef.current) return;

    isPollingRef.current = true;

    const checkJobs = async () => {
      console.log("Checking job status...");

      if (!shouldContinuePolling()) {
        console.log("No jobs to check, stopping polling");
        clearPolling();
        return;
      }

      try {
        let hasActiveJobs = false;

        // Check existing jobs from state
        if (jobs && Object.keys(jobs).length > 0) {
          for (const jobId in jobs) {
            const job: any = await webapp.get_job(BigInt(jobId));
            console.log(`Job ${jobId} status:`, job[0].status);

            if (job[0].status === "In Progress" || job[0].status === "Pending" || job[0].status === "Paused") {
              setJobs((prevJobs: any) => ({ ...prevJobs, [Number(jobId)]: job[0] }));
              hasActiveJobs = true;
            } else if (job[0].status === "Completed" || job[0].status === "Failed" || job[0].status === "Stopped") {
              setTimeout(() => removeJob(Number(jobId)), 3000);
            }
          }
        }

        // Check worker process jobs
        if (workerProcesses.length > 0) {
          for (const process of workerProcesses) {
            const job: any = await webapp.get_job(process.jobId);
            console.log(`Worker process job ${process.jobId} status:`, job[0].status);

            setJobs((prevJobs: any) => ({ ...prevJobs, [Number(process.jobId)]: job[0] }));

            if (job[0].status === "In Progress" || job[0].status === "Pending" || job[0].status === "Paused") {
              hasActiveJobs = true;
            } else if (job[0].status === "Completed" || job[0].status === "Failed" || job[0].status === "Stopped") {
              setTimeout(() => removeJob(Number(process.jobId)), 3000);
            }
          }
        }

        // Continue polling if there are active jobs
        if (hasActiveJobs || workerProcesses.length > 0) {
          timeoutRef.current = setTimeout(checkJobs, 1000) as unknown as number;
        } else {
          clearPolling();
        }
      } catch (error) {
        console.error("Error checking job status:", error);
        clearPolling();
      }
    };

    checkJobs();
  }, [webapp, jobs, workerProcesses, shouldContinuePolling, clearPolling, removeJob]);

  const stopJob = async (jobId: number) => {
    if (!webapp) return;

    setStopQueue((prevQueue) => [...prevQueue, jobId]);

    try {
      await webapp.stop_job(BigInt(jobId));
      console.log("Job stopped successfully");
    } catch (error) {
      console.error("Error stopping job:", error);
    }
  }

  const checkExistingJobs = async () => {
    if (!webapp || !address) {
      console.warn("No webapp or address found, cannot fetch existing jobs.");
      return;
    }

    try {
      const existingJobs: any = await webapp.get_job_by_owner(Principal.fromText(address));
      console.log("Existing jobs:", existingJobs);

      if (existingJobs && existingJobs.length > 0) {
        const jobsMap: { [key: number]: Job } = {};
        existingJobs.forEach((job: Job) => {
          if (job.status === "Stopped" || job.status === "Completed" || job.status === "Failed") return;
          jobsMap[Number(job.id)] = job;
        });
        setJobs(jobsMap);
        console.log("Jobs map:", jobsMap);
      } else {
        setJobs({});
      }
      setHasCheckedExistingJobs(true);
    } catch (error) {
      console.error("Error fetching existing jobs:", error);
      setJobs({});
      setHasCheckedExistingJobs(true);
    }
  }

  // Effect to check existing jobs on mount
  useEffect(() => {
    if (address && webapp && jobs === null && !hasCheckedExistingJobs) {
      console.log("Checking existing jobs on mount");
      checkExistingJobs();
    }
  }, [address, webapp, hasCheckedExistingJobs]);

  // Effect to start polling when jobs or worker processes change
  useEffect(() => {
    console.log("Jobs or worker processes changed:", { jobs, workerProcesses });

    clearPolling();

    if (shouldContinuePolling()) {
      console.log("Starting job status polling");
      queryJobStatus();
    }

    return () => {
      clearPolling();
    };
  }, [jobs, workerProcesses, queryJobStatus, shouldContinuePolling, clearPolling]);


  return (
    <nav className="flex justify-between mx-10 mb-12 mt-[1.5rem] items-center">
      <h1 className="text-2xl">
        <Link to={"/"}>FAI3</Link>
      </h1>
      <ul className="flex gap-12 items-center">
        <li>
          <Link to="/">Leaderboard</Link>
        </li>
        <li>
          <Link to="/">About</Link>
        </li>
        {
          connecting ? (
            <li>
              <CircularProgress />
            </li>
          ) : (
            <li className="border border-gray-300 rounded-md">
              <div className="flex items-center">
                {
                  webapp && authClient ? (
                    <>
                      {
                        jobs && Object.keys(jobs).length > 0 && (
                          <Popover>
                            <PopoverTrigger className="flex flex-row items-center justify-center p-2 text-sm gap-1">
                              <div className="relative group">
                                <div className="flex flex-row items-center justify-center p-2 text-sm gap-1">
                                  {Object.keys(jobs).length} <CircularProgress className="size-4" />
                                </div>
                              </div>
                            </PopoverTrigger>
                            <PopoverContent className="w-72 p-3">
                              <div className="flex flex-col gap-2">
                                <h3 className="text-sm font-semibold border-b pb-1.5">Running Tests</h3>
                                <ul className="flex flex-col gap-2">
                                  {Object.values(jobs).map((job: Job, index: number) => (
                                    <li key={index} className="flex items-center justify-between bg-slate-50 p-1.5 rounded-md text-xs">
                                      <div className="flex items-center gap-3">
                                        <div>
                                          <span className="text-gray-500 block">ID: {job.id.toString()}</span>
                                          <span className={`font-medium ${job.status === "In Progress" ? "text-blue-600" :
                                            job.status === "Completed" ? "text-green-600" :
                                              job.status === "Failed" ? "text-red-600" : ""
                                            }`}>
                                            {job.status}
                                          </span>
                                        </div>
                                        {/* Progress display */}
                                        <div className="ml-3 min-w-[48px] text-gray-700">
                                          {/* Replace job.progress.current and job.progress.total with your actual progress fields */}
                                          {job.progress ? (
                                            <span>{Number(job.progress.completed)}/{Number(job.progress.target)}</span>
                                          ) : (
                                            <span>-/-</span>
                                          )}
                                        </div>
                                        {/* Error count display */}
                                        <div className="ml-2 min-w-[48px] text-red-500">
                                          {/* Replace job.errorCount with your actual error count field */}
                                          {job.progress.call_errors || job.progress.invalid_responses ? (
                                            <span>{Number(job.progress.call_errors) + Number(job.progress.invalid_responses)} error{(Number(job.progress.call_errors) + Number(job.progress.invalid_responses)) !== 1 ? "s" : ""}</span>
                                          ) : (
                                            null
                                          )}
                                        </div>
                                      </div>
                                      {
                                        stopQueue.includes(Number(job.id)) ? (
                                          <CircularProgress className="size-4" />
                                        ) : (
                                          <Button
                                            onClick={() => stopJob(Number(job.id))}
                                            className="h-6 px-2 text-[10px]"
                                            variant="destructive"
                                          >
                                            Stop
                                          </Button>
                                        )
                                      }
                                    </li>
                                  ))}
                                </ul>
                              </div>
                            </PopoverContent>
                          </Popover>
                        )
                      }

                      <div className="relative group">
                        <p className="text-sm mx-2 cursor-pointer" onClick={copyAddress}>
                          {formatAddress(address)}
                        </p>
                        <span id="tooltip" className="absolute left-1/2 transform -translate-x-1/2 mb-2 px-2 py-1 text-xs text-white bg-black rounded opacity-0 transition-opacity duration-300">
                          Copied!
                        </span>
                      </div>
                      <Button onClick={disconnect}>Logout</Button>
                    </>
                  ) : (
                    <Button onClick={() => connect()}>Connect</Button>
                  )
                }
              </div>
            </li>
          )
        }
      </ul>
    </nav>
  );
}