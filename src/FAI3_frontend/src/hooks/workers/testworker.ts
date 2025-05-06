self.onmessage = async (event) => {
  try {
    const { data } = event;

    console.log("Worker received data:", data);

    self.postMessage({
      type: "start",
      data: data,
    });
  } catch (error) {
    console.error("Error in worker:", error);
    self.postMessage({
      type: "error",
      error: error,
    });
  }
};
