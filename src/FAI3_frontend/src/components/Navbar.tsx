import { Link } from "react-router-dom";
import { Actor, HttpAgent } from "@dfinity/agent";
import { Job } from "../../../declarations/FAI3_backend/FAI3_backend.did";
import { useEffect, useState, useContext } from "react";
import { Button, CircularProgress } from "./ui";
import { useAuthClient, formatAddress, useDataContext } from "../utils";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover"

export default function Navbar() {
  const { authClient, address, webapp, connect, disconnect, connecting } = useAuthClient();
  const { workerProcesses } = useDataContext();
  const [jobs, setJobs] = useState<{ [key: number]: Job } | null>(null);

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

  const queryJobStatus = async () => {
    if (!webapp) return;

    let timeoutId: number;

    const checkJobs = async () => {
      if (workerProcesses.length === 0) {
        clearTimeout(timeoutId);
        return;
      }

      console.log("Checking job status for worker processes:", workerProcesses);
      console.log("Current timeoutId:", timeoutId);

      for (let i = 0; i < workerProcesses.length; i++) {
        const process = workerProcesses[i];
        console.log("Checking job status for process:", process);
        try {
          const job: any = await webapp.get_job(process.jobId);
          console.log("Job:", job);

          setJobs((prevJobs: any) => ({ ...prevJobs, [Number(process.jobId)]: job[0] }));

          if (job[0].status === "Completed" || job[0].status === "Failed") {
            clearTimeout(timeoutId);
            setTimeout(() => {
              removeJob(Number(process.jobId));
            }, 3000);
            return;
          }
        } catch (error) {
          console.error("Error checking job status:", error);
          clearTimeout(timeoutId);
          setTimeout(() => {
            removeJob(Number(process.jobId));
          }, 3000);
          return;
        }
      }

      // Schedule the next check after 1 second
      clearTimeout(timeoutId);
      timeoutId = setTimeout(checkJobs, 1000) as unknown as number;
    };

    // Start the periodic checking
    checkJobs();

    return () => {
      clearTimeout(timeoutId);
    };
  }

  const removeJob = (jobId: number) => {
    setJobs((prevJobs: any) => {
      const newJobs = { ...prevJobs };
      delete newJobs[jobId];
      return newJobs;
    });
  }

  const stopJob = async (jobId: number) => {
    if (!webapp) return;

    try {
      await webapp.stop_job(BigInt(jobId));
      console.log("Job stopped successfully");
    } catch (error) {
      console.error("Error stopping job:", error);
    }
  }

  useEffect(() => {
    console.log("Worker processes:", workerProcesses);
    if (workerProcesses.length > 0) {
      queryJobStatus();
    }
    return () => {
      // Cleanup function to clear the timeout
    }
  }, [workerProcesses]);

  useEffect(() => {
    if (jobs) {
      console.log("Jobs:", jobs);
    }
  }, [jobs]);

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
                                  {workerProcesses.length} <CircularProgress className="size-4" />
                                </div>
                              </div>
                            </PopoverTrigger>
                            <PopoverContent className="w-72 p-3">
                              <div className="flex flex-col gap-2">
                                <h3 className="text-sm font-semibold border-b pb-1.5">Running Tests</h3>
                                <ul className="flex flex-col gap-2">
                                  {Object.values(jobs).map((job: Job, index: number) => (
                                    <li key={index} className="flex items-center justify-between bg-slate-50 p-1.5 rounded-md text-xs">
                                      <div className="flex items-center gap-2">
                                        <div>
                                          <span className="text-gray-500 block">ID: {job.id.toString()}</span>
                                          <span className={`font-medium ${job.status === "In Progress" ? "text-blue-600" :
                                            job.status === "Completed" ? "text-green-600" :
                                              job.status === "Failed" ? "text-red-600" : ""
                                            }`}>
                                            {job.status}
                                          </span>
                                        </div>
                                      </div>
                                      <Button
                                        onClick={() => stopJob(Number(job.id))}
                                        className="h-6 px-2 text-[10px]"
                                        variant="destructive"
                                      >
                                        Stop
                                      </Button>
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