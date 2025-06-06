let requestIdCounter = 0;
const pendingRequests = new Map();

// Handle messages from the main thread
self.onmessage = async (event) => {
  const { type, data, test } = event.data;

  if (type === 'INIT') {
    // Process the data
    try {
      let payload = {};

      console.log("Test type:", test);

      if (test === "CAT") {
        const { modelId, max_queries, seed, shuffle } = data;
        payload = {
          method: 'context_association_test',
          modelId,
          max_queries,
          seed,
          shuffle
        };
      } else if (test === "FAIRNESS") {
        const { modelId, max_queries, seed, dataset } = data;
        payload = {
          method: 'fairness_test',
          modelId,
          max_queries,
          seed,
          dataset
        };
      } else if (test === "KALEIDOSCOPE") {
        const { modelId, languages, max_queries, seed } = data;
        payload = {
          method: 'kaleidoscope_test',
          modelId,
          languages,
          max_queries,
          seed
        };
      } else {
        throw new Error("Unknown test type");
      }

      // Request the API call from the main thread
      const requestId = requestIdCounter++;

      const resultPromise = new Promise((resolve, reject) => {
        pendingRequests.set(requestId, { resolve, reject });
      });

      // Ask the main thread to make the API call
      self.postMessage({
        type: 'API_REQUEST',
        requestId,
        payload
      });

      // Wait for the result
      const result = await resultPromise;

      console.log("result", result);

      // Send the final result back
      self.postMessage({
        type: 'COMPLETE',
        payload: {
          success: true,
          data: result
        }
      });
    } catch (error: any) {
      self.postMessage({
        type: 'COMPLETE',
        payload: {
          success: false,
          error: error.message
        }
      });
    }
  } else if (type === 'API_RESPONSE') {
    // Handle API response
    const { requestId, success, data, error } = event.data;
    const pendingRequest = pendingRequests.get(requestId);

    if (pendingRequest) {
      if (success) {
        pendingRequest.resolve(data);
      } else {
        pendingRequest.reject(new Error(error));
      }
      pendingRequests.delete(requestId);
    }
  }
};
